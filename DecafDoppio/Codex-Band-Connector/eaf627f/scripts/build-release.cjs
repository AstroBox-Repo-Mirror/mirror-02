// Use the toolkit's standard release pipeline; never rewrite or re-sign its RPK.
const fs = require('fs'), path = require('path'), crypto = require('crypto'), cp = require('child_process');
const root = path.resolve(__dirname, '..');
const hash = b => crypto.createHash('sha256').update(b).digest('hex');
(async () => {
  require('../quickapp.config.js').signRequest();
  const toolkit = require('aiot-toolkit/package.json');
  if (toolkit.version !== '2.0.5') throw Error('aiot-toolkit must be 2.0.5; run npm ci.');
  const release = JSON.parse(fs.readFileSync(path.join(root, 'release.json'), 'utf8'));
  const source = JSON.parse(fs.readFileSync(path.join(root, 'src/manifest.json'), 'utf8'));
  for (const key of ['versionName', 'versionCode', 'name']) {
    if (source[key] !== release[key]) throw Error(`src/manifest.json ${key} must match release.json.`);
  }
  const relative = typeof toolkit.bin === 'string' ? toolkit.bin : toolkit.bin.aiot;
  const started = Date.now();
  const built = cp.spawnSync(process.execPath, [path.resolve(root, 'node_modules/aiot-toolkit', relative), 'release'], {cwd: root, encoding: 'utf8'});
  process.stdout.write(built.stdout || '');
  process.stderr.write(built.stderr || '');
  if (built.status !== 0 || !/build success/.test(built.stdout || '')) throw Error('Release compiler did not report success.');
  const target = path.join(root, 'dist', `${source.package}.release.${release.versionName}.rpk`);
  if (fs.statSync(target).mtimeMs < started - 2000) throw Error('Release output is stale.');
  const data = fs.readFileSync(target);
  const zip = await require('jszip').loadAsync(data);
  for (const name of ['manifest.json', 'manifest-watch.json']) {
    const manifest = JSON.parse(await zip.file(name).async('string'));
    for (const key of ['versionName', 'versionCode', 'name']) {
      if (manifest[key] !== release[key]) throw Error(`${name}: incorrect ${key}`);
    }
  }
  if (Object.keys(zip.files).some(name => name.endsWith('.map'))) throw Error('Unexpected source map in release.');
  const payloadDigests = {};
  for (const [name, entry] of Object.entries(zip.files)) {
    if (!entry.dir && !name.startsWith('META-INF/')) payloadDigests[name] = hash(await entry.async('nodebuffer'));
  }
  const certificate = fs.readFileSync(process.env.CODEX_BAND_SIGN_CERT);
  const report = {version: release.versionName, versionCode: release.versionCode, name: release.name,
    artifact: path.basename(target), sha256: hash(data), bytes: data.length,
    certificateFingerprint256: new crypto.X509Certificate(certificate).fingerprint256,
    payloadDigests, compiler: toolkit.version, node: process.version,
    sourceBuild: true, historicalRpkRead: false, standardRelease: true, command: 'aiot release'};
  fs.writeFileSync(path.join(root, 'dist/source-build.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({output: target, sha256: report.sha256, versionCode: report.versionCode}));
})().catch(error => { console.error(error.message); process.exitCode = 1; });
