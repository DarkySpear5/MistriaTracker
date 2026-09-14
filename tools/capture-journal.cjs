const { chromium } = require('C:/Users/ericd/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
(async()=>{
 const {spawn}=require('node:child_process');
 const path=require('node:path');
 const server=spawn(process.execPath,[path.resolve('node_modules/vite/bin/vite.js'),'--host','127.0.0.1','--port','1451','--strictPort'],{windowsHide:true,stdio:['ignore','pipe','pipe']});
 await new Promise((resolve,reject)=>{
  const timer=setTimeout(()=>reject(new Error('Preview server startup timed out')),10000);
  server.stdout.on('data',chunk=>{if(chunk.toString().includes('Local:')){clearTimeout(timer);resolve();}});
  server.once('error',error=>{clearTimeout(timer);reject(error);});
  server.once('exit',code=>{clearTimeout(timer);reject(new Error('Preview server exited '+code));});
 }).catch(error=>{server.kill();throw error;});
 const browser=await chromium.launch({headless:true,channel:'msedge'});
 try {
  const width=Number(process.argv[2])||1360;
  const suffix=width===1360?'':`-${width}`;
  const page=await browser.newPage({viewport:{width,height:width<1200?760:900}});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto('http://127.0.0.1:1451/tools/journal-visual.html');
  await page.getByRole('heading',{name:'Votre aventure à Mistria'}).waitFor();
  await page.screenshot({path:`diagnostics/journal-preview/overview${suffix}.png`});
  for(const [name,file] of [['Musée','museum'],['Villageois','villagers'],['Encyclopédie','encyclopedia']]){
   await page.locator('.sidebar nav').getByRole('button',{name:new RegExp(name)}).click();
   await page.waitForTimeout(600);
   await page.screenshot({path:`diagnostics/journal-preview/${file}${suffix}.png`});
  }
  const input=page.getByRole('combobox',{name:/Rechercher/});
  await input.fill('trui');
  const results=page.getByRole('listbox').getByRole('option');
  const matches=await results.count();
  if(matches){await results.first().click();await page.locator('.detail-panel').waitFor();await page.screenshot({path:`diagnostics/journal-preview/detail${suffix}.png`});}
  console.log(JSON.stringify({width,errors,matches,overflow:await page.evaluate(()=>document.documentElement.scrollWidth>window.innerWidth)}));
 } finally {await browser.close();server.kill();}
})();
