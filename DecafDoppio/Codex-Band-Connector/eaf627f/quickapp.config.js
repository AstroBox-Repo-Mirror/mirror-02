// Standard AIoT signing hook: keep the production identity outside the repository.
const fs = require('fs');
const crypto = require('crypto');
module.exports = {
  signRequest() {
    const keyPath = process.env.CODEX_BAND_SIGN_KEY;
    const certPath = process.env.CODEX_BAND_SIGN_CERT;
    if (!keyPath || !certPath) throw Error('Set CODEX_BAND_SIGN_KEY and CODEX_BAND_SIGN_CERT to external production PEM files.');
    const privatekey = fs.readFileSync(keyPath);
    const certificate = fs.readFileSync(certPath);
    const cert = new crypto.X509Certificate(certificate);
    if (cert.fingerprint256 !== require('./signing-identity.json').fingerprint256) throw Error('Certificate differs from the preserved production identity.');
    const challenge = Buffer.from('Codex Band Connector signing identity check');
    if (!crypto.verify('sha256', challenge, cert.publicKey, crypto.sign('sha256', challenge, privatekey))) throw Error('Private key does not match production certificate.');
    return {privatekey, certificate};
  }
};
