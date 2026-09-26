import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "./api";
import TechnicalFailures from "./TechnicalFailures";
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});
it("expande os motivos e pagina sem confundir arquivos com falhas", async () => {
  const user = userEvent.setup();
  const fetch = vi
    .spyOn(api, "technicalFailures")
    .mockResolvedValueOnce({
      total: 51,
      nextOffset: 50,
      items: [
        {
          assetId: "a",
          filename: "a.jpg",
          path: "D:/Fotos/a.jpg",
          previewError: "decoder failed",
          metadataError: "timeout",
        },
      ],
    })
    .mockResolvedValue({
      total: 51,
      nextOffset: null,
      items: [
        {
          assetId: "b",
          filename: "b.jpg",
          path: "D:/Fotos/b.jpg",
          previewError: "unsupported",
        },
      ],
    });
  render(<TechnicalFailures close={() => {}} />);
  await user.click(await screen.findByText("a.jpg"));
  expect(screen.getByText("decoder failed")).toBeVisible();
  expect(screen.getByText("timeout")).toBeVisible();
  await user.click(screen.getByRole("button", { name: "Próxima página" }));
  await waitFor(() => expect(fetch).toHaveBeenLastCalledWith(50));
  expect(await screen.findByText("b.jpg")).toBeVisible();
  expect(screen.getByRole("button", { name: "Próxima página" })).toBeDisabled();
});
it("mostra erro de consulta e permite tentar novamente", async () => {
  const user = userEvent.setup();
  vi.spyOn(api, "technicalFailures")
    .mockRejectedValueOnce(Error("Catálogo indisponível"))
    .mockResolvedValue({ total: 0, nextOffset: null, items: [] });
  render(<TechnicalFailures close={() => {}} />);
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Catálogo indisponível",
  );
  await user.click(screen.getByRole("button", { name: "Atualizar lista" }));
  expect(
    await screen.findByText("Nenhuma falha técnica nesta consulta."),
  ).toBeVisible();
});
