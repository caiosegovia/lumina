import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { api } from "./api";
import Discovery, { resetDiscoveryCache } from "./Discovery";

describe("descoberta local", () => {
  it("carrega grupos progressivamente e reutiliza snapshot da mesma biblioteca", async () => {
    const user = userEvent.setup();
    const base = await api.discovery();
    base.similar = Array.from({ length: 9 }, (_, i) => ({
      ...base.similar[0],
      id: `similar-${i}`,
    }));
    const fetch = vi.spyOn(api, "discovery").mockResolvedValue(base);
    const first = render(<Discovery navigate={() => {}} />);
    await screen.findByText("Redescubra sua biblioteca");
    expect(screen.getAllByText("Possível variação")).toHaveLength(4);
    await user.click(
      screen.getByRole("button", {
        name: /Mostrar mais em Visualmente parecidas/,
      }),
    );
    expect(screen.getAllByText("Possível variação")).toHaveLength(8);
    first.unmount();
    render(<Discovery navigate={() => {}} />);
    await screen.findByText("Redescubra sua biblioteca");
    expect(fetch).toHaveBeenCalledTimes(1);
    await user.click(
      screen.getByRole("button", { name: "Atualizar descobertas" }),
    );
    await waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
  });
  it("exibe operação ativa ao voltar para a seção e permite cancelar", async () => {
    const user = userEvent.setup();
    vi.spyOn(api, "discoveryWork").mockResolvedValue({
      running: true,
      stage: "Índice visual",
      completed: 2,
      total: 10,
      cancelled: false,
    });
    const cancel = vi.spyOn(api, "cancelDiscoveryWork").mockResolvedValue();
    render(<Discovery navigate={() => {}} />);
    expect(
      await screen.findByText("Índice visual: 2 de 10"),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Cancelar análise" }));
    expect(cancel).toHaveBeenCalledOnce();
  });
  afterEach(() => {
    cleanup();
    resetDiscoveryCache();
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
      await screen.findByRole("button", { name: "Recalcular lugares" }),
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
  it("faz curadoria reversível do melhor item de um burst", async () => {
    const user = userEvent.setup();
    const items = ["a", "b", "c"].map((id) => ({
      id,
      filename: `${id}.jpg`,
      mediaType: "photo" as const,
      capturedAt: "2026-01-02T14:30:00",
      camera: "Canon",
    }));
    vi.spyOn(api, "discovery").mockResolvedValue({
      indexed: 3,
      indexable: 3,
      similar: [],
      sequences: [
        {
          id: "burst-a",
          title: "Burst com 3 registros",
          detail: "Canon",
          score: 3,
          items,
          recommendedId: "b",
          recommendation: "Mais nítida",
        },
      ],
      memories: [],
      places: [],
      trips: [],
      locationStatus: { geotagged: 0, named: 0, approximate: 0 },
    });
    const update = vi
      .spyOn(api, "updateUserState")
      .mockResolvedValue({ affected: 1 });
    render(<Discovery navigate={() => {}} />);
    await user.click(
      await screen.findByRole("button", { name: "Manter melhor" }),
    );
    await waitFor(() => expect(update).toHaveBeenCalledTimes(2));
    expect(update).toHaveBeenNthCalledWith(1, {
      assetIds: ["b"],
      favorite: true,
    });
    expect(update).toHaveBeenNthCalledWith(2, {
      assetIds: ["a", "c"],
      reviewLater: true,
    });
    expect(await screen.findByText(/Nada foi excluído/)).toBeInTheDocument();
  });
  it("persiste perfis e permite revisar o burst inteiro", async () => {
    const user = userEvent.setup();
    const items = ["a", "b", "c"].map((id) => ({
      id,
      filename: `${id}.jpg`,
      mediaType: "photo" as const,
      capturedAt: "2026-01-02T14:30:00",
      camera: "Phone",
    }));
    vi.spyOn(api, "appPreferences").mockResolvedValue({
      resourceProfile: "balanced",
      curationRule: "review_all",
    });
    vi.spyOn(api, "discovery").mockResolvedValue({
      indexed: 3,
      indexable: 3,
      similar: [],
      sequences: [
        {
          id: "burst",
          title: "Burst",
          detail: "Phone",
          score: 3,
          items,
          recommendedId: "b",
        },
      ],
      memories: [],
      places: [],
      trips: [],
      locationStatus: { geotagged: 0, named: 0, approximate: 0 },
    });
    const save = vi
      .spyOn(api, "updateAppPreferences")
      .mockImplementation(async (value) => value);
    const update = vi
      .spyOn(api, "updateUserState")
      .mockResolvedValue({ affected: 3 });
    render(<Discovery navigate={() => {}} />);
    const profile = await screen.findByRole("combobox", {
      name: "Perfil de processamento",
    });
    await waitFor(() =>
      expect(
        screen.getByRole("combobox", { name: "Regra de curadoria" }),
      ).toHaveValue("review_all"),
    );
    await user.selectOptions(profile, "economy");
    await waitFor(() =>
      expect(save).toHaveBeenCalledWith({
        resourceProfile: "economy",
        curationRule: "review_all",
      }),
    );
    await user.click(screen.getByRole("button", { name: "Manter melhor" }));
    await waitFor(() =>
      expect(update).toHaveBeenCalledWith({
        assetIds: ["a", "b", "c"],
        reviewLater: true,
      }),
    );
    expect(
      await screen.findByText(/Burst inteiro enviado/),
    ).toBeInTheDocument();
  });
});
