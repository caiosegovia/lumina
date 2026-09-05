import { useEffect, useState } from "react";
import { Images, Layers3, LoaderCircle, RefreshCw, Search, Sparkles, Video } from "lucide-react";
import { api } from "./api";
import { openGalleryComparison, openGalleryWithFilters } from "./Gallery";
import type { DiscoveryGroup, DiscoveryItem, DiscoveryOverview, View } from "./types";
import "./discovery.css";
import "./discovery-actions.css";

function Thumb({item,onOpen,recommended=false}:{item:DiscoveryItem;onOpen:()=>void;recommended?:boolean}) {
  const [src,setSrc]=useState<string|null>();
  useEffect(()=>{let live=true;api.thumbnail(item.id).then(value=>live&&setSrc(value)).catch(()=>live&&setSrc(null));return()=>{live=false}},[item.id]);
  return <button className={`discovery-thumb ${recommended?"recommended":""}`} onClick={onOpen} title={item.filename}>{recommended&&<b>Melhor candidata</b>}{src?<img src={src} alt={`Prévia de ${item.filename}`}/>:<span>{item.mediaType==="video"?<Video/>:<Images/>}</span>}<small>{item.filename}</small></button>;
}

function Shelf({title,description,groups,empty,navigate}:{title:string;description:string;groups:DiscoveryGroup[];empty:string;navigate:(view:View)=>void}) {
  const open=(item:DiscoveryItem)=>{openGalleryWithFilters({query:item.filename});navigate("library")};
  const compare=(group:DiscoveryGroup)=>{openGalleryComparison(group.items.slice(0,2).map(item=>item.id));navigate("library")};
  return <section className="discovery-shelf"><div className="discovery-shelf-heading"><div><h3>{title}</h3><p>{description}</p></div><b>{groups.length}</b></div>{groups.length===0?<div className="discovery-empty">{empty}</div>:<div className="discovery-groups">{groups.map(group=><article className="discovery-group" key={group.id}><header><div><strong>{group.title}</strong><span>{group.detail}</span>{group.recommendation&&<small>{group.recommendation}</small>}</div><div className="discovery-group-actions">{group.score<=1&&<b>{Math.round(group.score*100)}%</b>}{group.items.length>=2&&<button onClick={()=>compare(group)}>Comparar</button>}</div></header><div className="discovery-strip">{group.items.map(item=><Thumb key={item.id} item={item} recommended={item.id===group.recommendedId} onOpen={()=>open(item)}/>)}</div></article>)}</div>}</section>;
}

export default function Discovery({navigate}:{navigate:(view:View)=>void}) {
  const [data,setData]=useState<DiscoveryOverview>(); const [busy,setBusy]=useState(false); const [message,setMessage]=useState(""); const [query,setQuery]=useState("");
  const load=()=>api.discovery().then(setData).catch(error=>setMessage(String(error)));
  useEffect(()=>{void load()},[]);
  const index=async()=>{setBusy(true);setMessage("Analisando imagens localmente…");try{const result=await api.buildDiscoveryIndex();setMessage(`${result.indexed} imagens analisadas · ${result.skipped} indisponíveis · ${result.failed} falhas`);await load()}catch(error){setMessage(String(error))}finally{setBusy(false)}};
  if(!data)return <div className="discovery-loading"><LoaderCircle className="spin"/>Preparando descobertas…</div>;
  const complete=data.indexable===0||data.indexed>=data.indexable;
  const filter=(groups:DiscoveryGroup[])=>{const term=query.trim().toLocaleLowerCase("pt-BR");return term?groups.filter(group=>`${group.title} ${group.detail} ${group.items.flatMap(item=>[item.filename,item.camera,...(item.visualLabels||[])]).join(" ")}`.toLocaleLowerCase("pt-BR").includes(term)):groups};
  return <div className="discovery"><div className="discovery-hero"><div><p className="eyebrow">DESCOBERTA LOCAL E EXPLICÁVEL</p><h2>Redescubra sua biblioteca</h2><p>Memórias, sequências e imagens parecidas identificadas somente neste computador.</p></div><button className="primary" disabled={busy||complete} onClick={index}>{busy?<LoaderCircle className="spin"/>:<RefreshCw/>}{complete?"Análise atualizada":"Analisar biblioteca"}</button></div>{message&&<div className="notice" role="status">{message}</div>}<div className="discovery-index"><Sparkles/><div><strong>{data.indexed.toLocaleString("pt-BR")} de {data.indexable.toLocaleString("pt-BR")} imagens analisadas</strong><span>{complete?"Índice visual atualizado":"A análise pode continuar enquanto você usa o Lumina"}</span></div><div className="discovery-index-bar"><i style={{width:`${data.indexable?data.indexed*100/data.indexable:100}%`}}/></div></div><label className="discovery-search"><Search/><input aria-label="Buscar nas descobertas" value={query} onChange={event=>setQuery(event.target.value)} placeholder="Buscar por arquivo, câmera, clara, escura, quente, fria…"/></label><Shelf title="Memórias" description="Registros deste período em outros anos." groups={filter(data.memories)} empty="As memórias aparecerão conforme sua linha do tempo crescer." navigate={navigate}/><Shelf title="Sequências" description="Rajadas e registros feitos no mesmo momento e equipamento." groups={filter(data.sequences)} empty="Nenhuma sequência com três ou mais registros foi encontrada." navigate={navigate}/><Shelf title="Visualmente parecidas" description="Sugestões por aparência; não são tratadas como duplicatas." groups={filter(data.similar)} empty="Execute a análise local para encontrar variações visuais." navigate={navigate}/><div className="discovery-safety"><Layers3/><div><strong>Você sempre decide</strong><p>Similaridade é apenas uma sugestão de curadoria. O Lumina não exclui, move nem altera seus originais.</p></div></div></div>;
}
