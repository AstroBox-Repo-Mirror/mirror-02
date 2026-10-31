const fs=require('fs'),path=require('path'),vm=require('vm'),assert=require('assert');
const root=path.resolve(__dirname,'..'),JSZip=require('jszip');
const candidate=path.join(root,'build'); const baseline=process.argv[2]; if(!baseline)throw Error('Pass the independently retained reference RPK path.');
function load(code){
 let now=1000,trace=[],connection={},timers=[];
 const apis={'system.interconnect':{instance:()=>connection},'system.storage':{set:x=>{trace.push(['persist',JSON.parse(x.value)]);x.success();},get:x=>x.success('{}')},'system.device':{getInfo:x=>x.success({osVersionName:'4.100.139'})},'system.app':{canIUse:()=>true,terminate:()=>trace.push(['terminate'])},'system.vibrator':{vibrate:x=>trace.push(['vibrate',x])}};
 connection.send=x=>trace.push(['send',x.data]);
 const runtime={Date:class extends Date {static now(){return now}},setTimeout:(f,d)=>{timers.push([f,d]);return timers.length},clearTimeout:i=>trace.push(['clearTimeout',i]),setInterval:(f,d)=>{trace.push(['interval',d]);return 123},clearInterval:i=>trace.push(['clearInterval',i]),$app_require$:id=>{const name=id.replace('@app-module/','').replace('@','');if(!apis[name])throw Error('Unknown native '+id);return apis[name]},console};
 const exports={};const ctx=vm.createContext({...runtime});
 const f=vm.runInContext('('+code.replace('export default','')+')',ctx);f(runtime,ctx,ctx,exports,()=>{});
 if(exports.entry)exports.entry(exports);
 return {component:exports.default,trace,connection,now:n=>now=n,flush:async()=>{await Promise.resolve();await Promise.resolve();},runtime,ctx};
}
function clean(value){return JSON.parse(JSON.stringify(value,(_,v)=>typeof v==='function'?undefined:v));}
function render(loaded, overrides){
 const component=loaded.component;
 const calls=[], state={...clean(component.data),...overrides};
 const page=new Proxy(state,{get:(o,k)=>k in o?o[k]:(...args)=>calls.push([k,...args])});
 const evaluate=x=>typeof x==='function'?x():x;
 const aiot={
  __ce__:(tag,config,children)=>{const options={};for(const [k,v] of Object.entries(config.__opts__)){if(k==='events'){options.events={};for(const [event,handler] of Object.entries(v)){const before=calls.length;handler('event');options.events[event]=calls.slice(before);}}else options[k]=evaluate(v);}return {tag,options,children:children.flat(Infinity).filter(Boolean)};},
  __ci__:(config,body)=>evaluate(config.__opts__.shown)?body():[],
  __cf__:(config,body)=>config.__opts__.exp().__list__.map((item,index)=>body(index,item))
 };
 loaded.ctx.aiot=aiot;
 loaded.ctx.$translateStyle$=x=>x;
 return clean(component.template(page,aiot));
}
async function scenario(code){
 const x=load(code),app=x.component;app.onCreate();const c=app.getBridge().controller;
 const stages=[];const snap=name=>stages.push({name,state:clean(c.state),trace:clean(x.trace)});
 snap('startup');x.connection.onopen();snap('transport-open');
 const ticket='t'.repeat(64),base={v:1,ticket,enabled:true,hookReady:false,safetyPaused:false,serverTime:1000};
 c.receive({...base,type:'hello',hostId:'pc',requests:[]});await x.flush();snap('hello');
 c.visible(true);c.receive({...base,type:'list',requests:[{id:'a',kind:'approval',pages:3,createdAt:1000}]});snap('new-approval');
 c.receive({...base,type:'detail',id:'a',kind:'approval',page:0,pages:3,content:'说明',presentation:'human',session:'s',canApprove:true,expiresAt:null});
 const allowed=c.decide('allow');snap('first-page-approval');assert.equal(allowed,true);
 c.receive({...base,type:'result',id:'a',state:'allow'});snap('resolved');
 c.receive({...base,type:'list',requests:[{id:'b',kind:'approval',pages:2,createdAt:1000},{id:'q',kind:'question',pages:1,createdAt:1000}]});
 c.open('b');c.page(1);snap('page-navigation');
 c.receive({...base,type:'detail',id:'b',kind:'approval',page:1,pages:2,content:'第二页',canApprove:true});c.decide('deny');snap('denied');
 c.receive({...base,type:'result',id:'b',state:'deny',source:'desktop'});snap('desktop-sync');
 c.visible(false);c.receive({...base,type:'list',requests:[{id:'q',kind:'question',pages:1,createdAt:1000}]});snap('background');
 c.toggleVibrate();c.toggleFullscreen();c.toggleAuto();await x.flush();snap('settings');
 c.transportFailure(204);snap('transport-failure');
 await c.forgetHost();await x.flush();snap('unbind');
 app.onDestroy();snap('destroy');return stages;
}
(async()=>{
 const zip=await JSZip.loadAsync(fs.readFileSync(baseline));
 const baselineApp=await zip.file('app.js').async('string'),rebuiltApp=fs.readFileSync(path.join(candidate,'app.js'),'utf8');
 const original=await scenario(baselineApp),rebuilt=await scenario(rebuiltApp);
 fs.writeFileSync(path.join(root,'dist/controller-traces.json'),JSON.stringify({original,rebuilt},null,2));
 assert.deepEqual(rebuilt,original);
 const originalLoaded=load(await zip.file('pages/index/index.js').async('string')),rebuiltLoaded=load(fs.readFileSync(path.join(candidate,'pages/index/index.js'),'utf8'));
 const originalPage=originalLoaded.component,rebuiltPage=rebuiltLoaded.component;
 assert.deepEqual(clean(rebuiltPage.style),clean(originalPage.style));
 assert.deepEqual(clean(rebuiltPage.data),clean(originalPage.data));
 const cases=[{view:'inbox',requests:[],hint:''},{view:'inbox',requests:[{id:'a',label:'请求A',time:'12:00',project:'项目'},{id:'b',label:'请求B',time:'12:01',project:'项目2'}],hint:'电脑已连接'},...['detail'].flatMap(view=>[true,false].flatMap(isApproval=>[true,false].map(canApprove=>({view,isApproval,canApprove,content:'测试内容',contentHeight:500,pageLabel:'1 / 3'})))),...[true,false].flatMap(enabled=>[true,false].map(bindConfirm=>({view:'settings',enabled,bindConfirm,hostBound:true,fixtureMode:false}))),{view:'settings',fixtureMode:true}];
 for(const state of cases){const a=render(rebuiltLoaded,state),b=render(originalLoaded,state);if(process.argv.includes('--approval-no-pager')&&state.view==='detail'&&state.isApproval){const detail=b.children[0];const pager=detail.children.findIndex(n=>n.options.classList.includes('pager'));assert(pager>=0,'Baseline approval pager missing');detail.children.splice(pager,1);assert(!a.children[0].children.some(n=>n.options.classList.includes('pager')),'Approval pager still rendered');}if(process.argv.includes('--question-no-pager')&&state.view==='detail'&&!state.isApproval){const detail=b.children[0];const pager=detail.children.findIndex(n=>n.options.classList.includes('pager'));assert(pager>=0,'Baseline question pager missing');detail.children.splice(pager,1);assert(!a.children[0].children.some(n=>n.options.classList.includes('pager')),'Question pager still rendered');}if(JSON.stringify(a)!==JSON.stringify(b)){fs.writeFileSync(path.join(root,'dist/render-diff.json'),JSON.stringify({state,a,b},null,2));}assert.deepEqual(a,b);}
 console.log('Controller/native/lifecycle traces identical:',original.length,'stages. Page styles and defaults identical.');
 console.log('Rendered trees and click targets identical:',cases.length,'page scenarios.');
})();
