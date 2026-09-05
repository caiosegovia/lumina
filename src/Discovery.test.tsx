import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { api } from "./api";
import Discovery from "./Discovery";

describe("descoberta local",()=>{
  afterEach(()=>{cleanup();vi.restoreAllMocks()});
  it("explica sugestões sem confundi-las com duplicatas",async()=>{render(<Discovery navigate={()=>{}}/>);expect(await screen.findByText("Redescubra sua biblioteca")).toBeInTheDocument();expect(screen.getByText("Visualmente parecidas")).toBeInTheDocument();expect(screen.getByText(/não exclui, move nem altera/)).toBeInTheDocument();expect(screen.getByText("94%")).toBeInTheDocument()});
  it("constrói o índice local e atualiza o progresso",async()=>{const user=userEvent.setup();vi.spyOn(api,"discovery").mockResolvedValueOnce({indexed:0,indexable:2,similar:[],sequences:[],memories:[]}).mockResolvedValueOnce({indexed:2,indexable:2,similar:[],sequences:[],memories:[]});const build=vi.spyOn(api,"buildDiscoveryIndex").mockResolvedValue({indexed:2,skipped:0,failed:0});render(<Discovery navigate={()=>{}}/>);await user.click(await screen.findByRole("button",{name:"Analisar biblioteca"}));await waitFor(()=>expect(build).toHaveBeenCalledOnce());expect(await screen.findByText("2 de 2 imagens analisadas")).toBeInTheDocument();expect(screen.getByRole("button",{name:"Análise atualizada"})).toBeDisabled()});
});
