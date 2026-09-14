import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { api } from "./api";
import Discovery from "./Discovery";

describe("descoberta local", () => {
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });
  it("explica sugestões sem confundi-las com duplicatas", async () => {
    render(<Discovery navigate={() => {}} />);
    expect(
      await screen.findByText("Redescubra sua biblioteca"),
    ).toBeInTheDocument();
    expect(screen.getByText("Visualmente parecidas")).toBeInTheDocument();
    expect(screen.getByText(/não exclui, move nem altera/)).toBeInTheDocument();
    expect(screen.getByText("94%")).toBeInTheDocument();
  });
  it("constrói o índice local e atualiza o progresso", async () => {
    const user = userEvent.setup();
    const locationStatus = { geotagged: 0, named: 0, approximate: 0 };
    vi.spyOn(api, "discovery")
      .mockResolvedValueOnce({
        indexed: 0,
        indexable: 2,
        similar: [],
        sequences: [],
        memories: [],
        places: [],
        trips: [],
        locationStatus,
      })
      .mockResolvedValueOnce({
        indexed: 2,
        indexable: 2,
        similar: [],
        sequences: [],
        memories: [],
        places: [],
        trips: [],
        locationStatus,
      });
    const build = vi
      .spyOn(api, "buildDiscoveryIndex")
      .mockResolvedValue({ indexed: 2, skipped: 0, failed: 0 });
    render(<Discovery navigate={() => {}} />);
    await user.click(
      await screen.findByRole("button", { name: "Analisar biblioteca" }),
    );
    await waitFor(() => expect(build).toHaveBeenCalledOnce());
    expect(
      await screen.findByText("2 de 2 imagens analisadas"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Análise atualizada" }),
    ).toBeDisabled();
  });
  it("resolve e permite corrigir nomes de lugares sem alterar originais", async () => {
    const user = userEvent.setup();
    const place = {
      id: "place-1",
      title: "São Paulo, SP · Brasil",
      detail: "1 registro com localização",
      score: 1,
      items: [],
      placeKey: "-23.55:-46.65",
    };
    vi.spyOn(api, "discovery").mockResolvedValue({
      indexed: 0,
      indexable: 0,
      similar: [],
      sequences: [],
      memories: [],
      places: [place],
      trips: [],
      locationStatus: { geotagged: 1, named: 1, approximate: 0 },
    });
    const resolve = vi
      .spyOn(api, "resolveLocationNames")
      .mockResolvedValue({ resolved: 1, named: 1, approximate: 0 });
    const rename = vi.spyOn(api, "renameLocation").mockResolvedValue();
    render(<Discovery navigate={() => {}} />);
    await user.click(
      await screen.findByRole("button", { name: "Nomear lugares" }),
    );
    await waitFor(() => expect(resolve).toHaveBeenCalledOnce());
    await user.click(screen.getByRole("button", { name: "Renomear" }));
    const input = screen.getByRole("textbox", {
      name: "Nome personalizado do lugar",
    });
    await user.clear(input);
    await user.type(input, "Casa");
    await user.click(screen.getByRole("button", { name: "Salvar nome" }));
    await waitFor(() =>
      expect(rename).toHaveBeenCalledWith("-23.55:-46.65", "Casa"),
    );
    expect(await screen.findByText(/somente no catálogo/)).toBeInTheDocument();
  });
});
