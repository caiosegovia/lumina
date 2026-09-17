import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import ActivityCenter from "./ActivityCenter";
import { api } from "./api";
import type { JobOverview } from "./types";

afterEach(()=>{cleanup();vi.restoreAllMocks()});

const pending:JobOverview={jobId:"job-protection",sourceName:"Importação",sourcePath:"E:\\DCIM",state:"protection_pending",stage:"protection_pending",processedItems:10,totalItems:10,processedBytes:100,totalBytes:100,overallPercent:100,imported:10,duplicates:0,excluded:0,failed:0,createdAt:"2026-09-01T00:00:00Z",updatedAt:"2026-09-01T00:01:00Z"};

it("executa a proteção em vez de apenas abrir os detalhes",async()=>{
  vi.spyOn(api,"events").mockResolvedValue([]);
  vi.spyOn(api,"backgroundWork").mockResolvedValue([]);
  vi.spyOn(api,"getLibrary").mockResolvedValue(null);
  const protect=vi.spyOn(api,"startProtection").mockResolvedValue();
  const open=vi.fn();
  render(<ActivityCenter jobs={[pending]} openJob={open}/>);
  await userEvent.click(screen.getByRole("button",{name:"Proteger agora"}));
  await waitFor(()=>expect(protect).toHaveBeenCalledWith("job-protection"));
  expect(open).not.toHaveBeenCalled();
  expect(screen.getByRole("status")).toHaveTextContent("Proteção adicionada à fila");
});

it("separa execução, atenção e histórico sem misturar os estados",async()=>{
  vi.spyOn(api,"events").mockResolvedValue([]);
  vi.spyOn(api,"backgroundWork").mockResolvedValue([]);
  vi.spyOn(api,"getLibrary").mockResolvedValue(null);
  const active={...pending,jobId:"active",sourceName:"Cartão ativo",state:"analyzing",stage:"metadata",overallPercent:35};
  const completed={...pending,jobId:"done",sourceName:"Importação concluída",state:"completed",stage:"completed"};
  render(<ActivityCenter jobs={[active,pending,completed]} openJob={vi.fn()}/>);
  await userEvent.click(screen.getByRole("button",{name:/Em execução/}));
  expect(screen.getByRole("progressbar",{name:"Progresso de Cartão ativo"})).toBeInTheDocument();
  expect(screen.queryByText("Importação concluída")).not.toBeInTheDocument();
  await userEvent.click(screen.getByRole("button",{name:/Atenção/}));
  expect(screen.getByRole("button",{name:"Proteger agora"})).toBeInTheDocument();
  expect(screen.queryByRole("progressbar",{name:"Progresso de Cartão ativo"})).not.toBeInTheDocument();
  await userEvent.click(screen.getByRole("button",{name:/Histórico/}));
  expect(screen.getAllByText("Importação concluída").length).toBeGreaterThan(0);
  expect(screen.queryByRole("button",{name:"Proteger agora"})).not.toBeInTheDocument();
});
