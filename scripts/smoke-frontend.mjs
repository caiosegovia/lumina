// Functional + geometry review. Run against Vite; packaged tests use real IPC separately.
import { chromium, expect } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import assert from 'node:assert/strict';
const root='artifacts/0.29.1/frontend';
mkdirSync(root,{recursive:true});
const server=process.argv.includes('--serve')?await (await import('vite')).createServer({server:{host:'127.0.0.1',port:1420,strictPort:true}}):undefined;
await server?.listen();
const browser=await chromium.launch({...process.env.CI?{}:{channel:'msedge'},headless:true});
try {
  for(const [width,height,scale] of [[1400,900,1],[1100,700,1.25],[1000,700,1.5],[800,560,1.25],[667,467,1.5]]) {
    if(process.env.SMOKE_NARROW && width>=1000)continue;
    for(const theme of ['light','dark']) {
      const page=await browser.newPage({viewport:{width,height},deviceScaleFactor:scale});
      const errors=[];
      page.on('pageerror',e=>errors.push(String(e)));
      await page.goto('http://127.0.0.1:1420');
      await page.evaluate(theme=>{localStorage.setItem('lumina-theme',theme);document.documentElement.dataset.theme=theme},theme);
      await page.getByLabel('Pasta-mestre').fill('D:\\Synthetic\\Master');
      await page.getByLabel('Pasta de backup').fill('E:\\Synthetic\\Replica');
      await page.getByRole('button',{name:/Criar biblioteca/}).click();
      await page.locator('aside nav').waitFor();
      const geometry=async(label)=>{
        const sizes=await page.evaluate(()=>({viewport:innerWidth,document:document.documentElement.scrollWidth,form:document.querySelector('.gallery-search')?.getBoundingClientRect().height}));
        if(sizes.document>sizes.viewport+1) {
          console.log(await page.evaluate(()=>[...document.querySelectorAll('main *')].filter(e=>e.getBoundingClientRect().right>innerWidth).map(e=>({tag:e.tagName,class:e.className,right:e.getBoundingClientRect().right})).slice(0,25)));
          await page.screenshot({path:`${root}/overflow-${width}-${theme}.png`,fullPage:true});
        }
        assert(sizes.document<=sizes.viewport+1,`${label}: page overflow ${JSON.stringify(sizes)}`);
        if(sizes.form)assert(sizes.form<=50,`${label}: search height ${sizes.form}`);
      };
      for(const section of ['Visão geral','Biblioteca','Descobrir','Revisão','Fontes','Duplicatas','Álbuns','Atividade','Proteção']) {
        const nav=page.locator('aside nav').getByRole('button',{name:new RegExp(`^${section}`)});
        if(!await nav.count())throw Error(`Navigation missing: ${section}`);
        await nav.click();
        await page.waitForTimeout(400);
        await geometry(section);
        await page.screenshot({path:`${root}/${width}-${theme}-${section}.png`,fullPage:true});
      }
      await page.getByRole('button',{name:'Biblioteca',exact:true}).click();
      await page.getByRole('button',{name:'Abrir detalhes de IMG_2401.JPG',exact:true}).waitFor();
      await page.evaluate(async()=>{
        const {api}=await import('/src/api.ts');
        const original=api.gallery;
        api.gallery=async(...args)=>{await new Promise(r=>setTimeout(r,args[0].query==='Canon'?900:200));return original(...args)};
      });
      const input=page.getByLabel('Buscar na galeria');
      await input.fill('Canon');
      await input.press('Enter');
      await expect(page.locator('.gallery-query-status')).toContainText('Buscando');
      await geometry('loading');
      await page.screenshot({path:`${root}/${width}-${theme}-loading.png`});
      await input.fill('DJI');
      await page.getByRole('button',{name:'Buscar',exact:true}).click();
      await expect(page.locator('.gallery-query-status')).toContainText('4 resultados');
      await page.waitForTimeout(1000);
      await expect(page.locator('.gallery-query-status')).toContainText('4 resultados');
      await page.getByRole('button',{name:/^Fotos/}).click();
      await expect(page.locator('.gallery-query-status')).toContainText('3 resultados');
      await page.getByRole('button',{name:'Remover filtro Fotos',exact:true}).click();
      await expect(page.locator('.gallery-query-status')).toContainText('4 resultados');
      await input.fill('not-a-real-file'); await input.press('Enter');
      await expect(page.getByRole('heading',{name:'Nenhum resultado encontrado'})).toBeVisible();
      await page.getByRole('button',{name:'Limpar busca e filtros'}).click();
      await expect(page.locator('.gallery-query-status')).toContainText('18 resultados');
      await page.getByRole('button',{name:'Visão em lista'}).click();
      await geometry('list');
      await page.getByRole('button',{name:'Abrir detalhes de IMG_2401.JPG',exact:true}).click();
      await geometry('inspector');
      await page.screenshot({path:`${root}/${width}-${theme}-inspector.png`});
      await page.getByRole('button',{name:'Fechar detalhes',exact:true}).click();
      await page.keyboard.press('Control+f'); await expect(input).toBeFocused();
      await input.fill('DJI'); await input.press('Enter');
      await expect(page.locator('.gallery-query-status')).toContainText('4 resultados');
      await input.press('Escape');
      await expect(page.locator('.gallery-query-status')).toContainText('18 resultados');
      assert.deepEqual(errors,[]);
      console.log(`PASS frontend + search ${width}x${height} DPR ${scale} ${theme}`);
      await page.close();
    }
  }
} finally {await browser.close();await server?.close()}
