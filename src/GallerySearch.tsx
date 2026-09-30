import type { FormEvent, RefObject } from 'react';
import { Search, X } from 'lucide-react';

export default function GallerySearch({inputRef,value,hasQuery,busy,change,submit,clear}:{
  inputRef:RefObject<HTMLInputElement>;value:string;hasQuery:boolean;busy:boolean;
  change:(value:string)=>void;submit:(event:FormEvent)=>void;clear:()=>void;
}) {
  return <form className="search gallery-search" role="search" aria-busy={busy} onSubmit={submit}>
    <Search/>
    <input ref={inputRef} aria-label="Buscar na galeria" placeholder="Nome, equipamento, tag, álbum ou lugar"
      value={value} maxLength={300} onChange={event=>change(event.target.value)}
      onKeyDown={event=>{if(event.key==='Escape'&&(value||hasQuery)){event.preventDefault();event.stopPropagation();clear()}}}/>
    {(value||hasQuery)&&<button type="button" className="search-clear" aria-label="Limpar busca" onClick={clear}><X/></button>}
    <button className="primary search-submit" aria-label="Buscar" type="submit"><Search/><span>Buscar</span></button>
  </form>;
}
