import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { api } from "./api";
import type { DashboardStats } from "./types";
const emptyDashboard=():DashboardStats=>({totalAssets:0,photos:0,videos:0,bytes:0,protected:0,pending:0,duplicateGroups:0,duplicateBytes:0,reclaimableBytes:0,errors:0,offlineSources:0,masterAvailableBytes:0,backupAvailableBytes:0,types:[],years:[],months:[],protection:[],cameras:[],formats:[],sources:[],insights:[],snapshotGeneratedAt:new Date().toISOString(),stale:false,timings:[]});

describe("fluxo principal do aplicativo", () => {
  afterEach(() => { cleanup(); vi.restoreAllMocks(); });
  it("cria a biblioteca, abre o painel e conclui o assistente de importação", async () => {
    const user = userEvent.setup();
    render(<App />);

    expect(await screen.findByText("Crie sua biblioteca")).toBeInTheDocument();
    const master = screen.getByLabelText("Pasta-mestre");
    const backup = screen.getByLabelText("Pasta de backup");
    await user.clear(master);
    await user.type(master, "D:\\Lumina\\Originais");
    await user.clear(backup);
    await user.type(backup, "G:\\Meu Drive\\Lumina Backup");
    await user.click(screen.getByRole("button", { name: /Criar biblioteca/ }));

    expect(await screen.findByText(/memórias ·/)).toBeInTheDocument();
    expect(screen.getByText("PROTEÇÃO E ESPAÇO")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Importar mídia" }));
    expect(await screen.findByText("Escolha uma fonte")).toBeInTheDocument();
    const chooseFolder = vi.spyOn(api, "chooseFolder").mockResolvedValue("E:\\Fotos da câmera");
    await user.click(screen.getByRole("button", { name: /Escolher pasta/ }));
    expect(chooseFolder).toHaveBeenCalledOnce();
    expect(screen.getByDisplayValue("E:\\Fotos da câmera")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Analisar fonte/ }));
    expect(await screen.findByText("Pronto para consolidar")).toBeInTheDocument();
    expect(screen.getByText("391")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Consolidar 391 itens/ }));
    expect(await screen.findByRole("progressbar", { name: "Progresso geral da importação" })).toBeInTheDocument();
    expect(screen.getByRole("progressbar", { name: "Progresso da etapa" })).toBeInTheDocument();
    expect(screen.getByText(/Acervo:/)).toBeInTheDocument();
    expect(screen.getByText(/Backup:/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Pausar" }));
    expect(await screen.findByText("Importação pausada")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Retomar" }));
    expect(await screen.findByText("Suas mídias foram verificadas.", {}, { timeout: 5000 })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ver atividade" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Ver mídias importadas" }));
    await waitFor(() => expect(screen.queryByText("IMPORTAÇÃO CONCLUÍDA")).not.toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: "Biblioteca" }));
    const mediaName = await screen.findByText("IMG_2401.JPG");
    await user.click(mediaName);
    expect(await screen.findByText("SHA-256")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Fechar detalhes" }));
    const search = screen.getByPlaceholderText("Buscar por nome, câmera ou tag…");
    await user.type(search, "DJI");
    await waitFor(() => expect(screen.queryByText("IMG_2401.JPG")).not.toBeInTheDocument());

    await user.click(screen.getByRole("button", { name: "Fontes" }));
    expect(await screen.findByText("De onde vêm suas mídias")).toBeInTheDocument();
    expect(await screen.findByText("HD Fotos Antigas")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Duplicatas" }));
    expect(await screen.findByText("Duplicatas exatas")).toBeInTheDocument();
    expect(await screen.findAllByText("3 cópias")).toHaveLength(2);

    await user.click(screen.getByRole("button", { name: "Álbuns" }));
    expect(await screen.findByText("Viagens")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Atividade" }));
    expect(await screen.findByText(/Análise concluída/)).toBeInTheDocument();
    await user.click(screen.getByText("Diagnósticos e relatórios"));
    await user.click(screen.getByRole("button", { name: "Exportar relatório completo" }));
    expect(await screen.findByRole("status")).toHaveTextContent("report.jsonl");

    await user.click(screen.getByRole("button", { name: "Proteção" }));
    expect(await screen.findByText("Proteção e armazenamento")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Verificar agora/ }));
    expect(await screen.findByText("Verificação iniciada em segundo plano. Acompanhe, pause ou cancele em Atividade.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Limpar cache" }));
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("18 miniaturas removidas"));
    await user.click(screen.getByRole("button", { name: "Reconstruir miniaturas" }));
    await waitFor(() => expect(screen.getByRole("status")).toHaveTextContent("18 miniaturas geradas · 0 falhas"));

    const failure = vi.spyOn(api, "startAnalysis").mockRejectedValueOnce(new Error("Fonte indisponível"));
    await user.click(screen.getByRole("button", { name: /Nova importação/ }));
    await user.click(screen.getByRole("button", { name: /Analisar fonte/ }));
    expect(await screen.findByText(/Fonte indisponível/)).toBeInTheDocument();
    failure.mockRestore();
  }, 15000);

  it("oferece retomar ou descartar um trabalho interrompido", async () => {
    const user = userEvent.setup();
    vi.spyOn(api, "getLibrary").mockResolvedValue({id:"lib",name:"Teste",masterPath:"D:\\Lumina",backupPath:"G:\\Backup",createdAt:new Date().toISOString()});
    vi.spyOn(api, "dashboard").mockResolvedValue(emptyDashboard());
    vi.spyOn(api, "recoverableJobs").mockResolvedValue([{jobId:"job-1",sourcePath:"E:\\DCIM",state:"interrupted",stage:"hashing",interruptionReason:"Fechamento inesperado",updatedAt:new Date().toISOString()}]);
    const discard = vi.spyOn(api, "discardJob").mockResolvedValue();
    render(<App/>);
    expect(await screen.findByText("Uma importação foi interrompida")).toBeInTheDocument();
    expect(screen.getByText("Fechamento inesperado")).toBeInTheDocument();
    await user.click(screen.getByRole("button",{name:"Descartar trabalho"}));
    await waitFor(()=>expect(discard).toHaveBeenCalledWith("job-1"));
    expect(screen.queryByText("Uma importação foi interrompida")).not.toBeInTheDocument();
  });

  it("cancela uma consolidação e mantém um diagnóstico compreensível", async () => {
    const user=userEvent.setup();
    vi.spyOn(api,"getLibrary").mockResolvedValue({id:"lib",name:"Teste",masterPath:"D:\\Lumina",backupPath:"G:\\Backup",createdAt:new Date().toISOString()});
    vi.spyOn(api,"dashboard").mockResolvedValue(emptyDashboard());
    vi.spyOn(api,"recoverableJobs").mockResolvedValue([]);
    render(<App/>);
    await screen.findByText(/memórias ·/);
    await user.click(screen.getByRole("button",{name:/Nova importação/}));
    await user.click(screen.getByRole("button",{name:/Analisar fonte/}));
    await screen.findByText("Pronto para consolidar");
    await user.click(screen.getByRole("button",{name:/Consolidar 391 itens/}));
    await screen.findByRole("progressbar",{name:"Progresso geral da importação"});
    await user.click(screen.getByRole("button",{name:"Cancelar importação"}));
    expect(await screen.findByText(/Importação cancelada/)).toBeInTheDocument();
    expect(screen.getByText(/Arquivos já verificados permanecem seguros/)).toBeInTheDocument();
  },10000);

  it("permite fechar a análise e continuar navegando", async()=>{
    const user=userEvent.setup();
    vi.spyOn(api,"getLibrary").mockResolvedValue({id:"lib",name:"Teste",masterPath:"D:\\Lumina",backupPath:"G:\\Backup",createdAt:new Date().toISOString()});
    vi.spyOn(api,"dashboard").mockResolvedValue(emptyDashboard());
    vi.spyOn(api,"recoverableJobs").mockResolvedValue([]);
    render(<App/>);
    await screen.findByText(/memórias ·/);
    await user.click(screen.getByRole("button",{name:/Nova importação/}));
    await user.click(screen.getByRole("button",{name:/Analisar fonte/}));
    const continueButton=await screen.findByRole("button",{name:"Continuar navegando"});
    await user.click(continueButton);
    expect(screen.queryByText("Analisando em segundo plano…")).not.toBeInTheDocument();
    expect(screen.getByText(/memórias ·/)).toBeInTheDocument();
  });
});
