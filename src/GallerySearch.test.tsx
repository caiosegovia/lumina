import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, expect, it, vi } from 'vitest';
import Gallery,{resetGallerySession,openGalleryWithFilters} from './Gallery';
import {api} from './api';
import type {GalleryResult} from './types';

afterEach(()=>{cleanup();resetGallerySession();localStorage.clear();vi.restoreAllMocks()});
it('sincroniza o campo ao aplicar uma visão salva',async()=>{
  vi.spyOn(api,'savedViews').mockResolvedValue([{id:'saved',name:'Drones',filters:{query:'DJI'},smartAlbum:false,createdAt:'',updatedAt:''}]);
  render(<Gallery/>);
  fireEvent.change(await screen.findByLabelText('Visões salvas'),{target:{value:'saved'}});
  await waitFor(()=>expect(screen.getByLabelText('Buscar na galeria')).toHaveValue('DJI'));
  expect(await screen.findByRole('button',{name:'Remover filtro Busca: “DJI”'})).toBeInTheDocument();
});
it('não confunde cópia mestre com réplica e mostra ausência de GPS',async()=>{
  openGalleryWithFilters({protectionState:'consolidated',hasLocation:false});
  render(<Gallery/>);
  expect(screen.getByRole('button',{name:'Remover filtro No acervo mestre'})).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Remover filtro Sem localização'})).toBeInTheDocument();
});
it('ignora resposta obsoleta e mantém uma busca nova disponível durante carregamento',async()=>{
  const all=await api.gallery({query:''});
  let first!:(result:GalleryResult)=>void;
  let second!:(result:GalleryResult)=>void;
  const spy=vi.spyOn(api,'gallery').mockImplementation(filters=>filters.query==='old'?new Promise(resolve=>{first=resolve}):filters.query==='new'?new Promise(resolve=>{second=resolve}):Promise.resolve(all));
  render(<Gallery/>);
  await screen.findByText('IMG_2401.JPG');
  const input=screen.getByLabelText('Buscar na galeria');
  fireEvent.change(input,{target:{value:'old'}});fireEvent.submit(screen.getByRole('search'));
  await waitFor(()=>expect(spy).toHaveBeenCalledWith(expect.objectContaining({query:'old'}),undefined,100,'captured_desc'));
  expect(screen.getByRole('search')).not.toHaveClass('loading');
  expect(screen.getByRole('button',{name:'Buscar'})).toBeEnabled();
  fireEvent.change(input,{target:{value:'new'}});fireEvent.submit(screen.getByRole('search'));
  await waitFor(()=>expect(spy).toHaveBeenCalledWith(expect.objectContaining({query:'new'}),undefined,100,'captured_desc'));
  await act(async()=>second({...all,assets:[],matched:0,nextCursor:undefined}));
  expect(screen.getByText('Nenhum resultado encontrado')).toBeInTheDocument();
  await act(async()=>first(all));
  expect(screen.getByText('Nenhum resultado encontrado')).toBeInTheDocument();
  expect(screen.queryByText('IMG_2401.JPG')).not.toBeInTheDocument();
});

it('permite repetir a mesma consulta após erro',async()=>{
  const all=await api.gallery({query:''});
  const spy=vi.spyOn(api,'gallery').mockResolvedValueOnce(all).mockRejectedValueOnce(new Error('Falha temporária')).mockResolvedValue(all);
  render(<Gallery/>);await screen.findByText('IMG_2401.JPG');
  fireEvent.change(screen.getByLabelText('Buscar na galeria'),{target:{value:'IMG'}});
  fireEvent.submit(screen.getByRole('search'));
  await screen.findByText('Error: Falha temporária');
  fireEvent.click(screen.getByRole('button',{name:'Buscar'}));
  await waitFor(()=>expect(spy).toHaveBeenCalledTimes(3));
  await waitFor(()=>expect(screen.queryByText('Error: Falha temporária')).not.toBeInTheDocument());
});
