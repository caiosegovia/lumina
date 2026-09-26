import { useEffect, useState } from "react";
import { api } from "./api";
import type { FailurePage } from "./types";
import { formatDate } from "./format";

function guidance(error: string) {
  const message = error.toLocaleLowerCase();
  if (/sem pr[eé]via|unsupported|não suport|corromp|decode/.test(message))
    return "Possível limitação do formato ou arquivo ilegível. Confira o original; repetir o processamento pode não resolver.";
  if (/not found|não está disponível|os error 2|inacessível/.test(message))
    return "Confira se a unidade está conectada e o arquivo existe no caminho indicado.";
  if (/timeout|tempo limite|busy/.test(message))
    return "Uma nova tentativa pode resolver após os outros trabalhos terminarem.";
  return "Consulte o motivo registrado. Se persistir, exporte o diagnóstico para análise; a causa ainda não está classificada.";
}
export default function TechnicalFailures({ close }: { close: () => void }) {
  const [page, setPage] = useState<FailurePage>(),
    [offset, setOffset] = useState(0),
    [refresh, setRefresh] = useState(0),
    [loading, setLoading] = useState(true),
    [error, setError] = useState(""),
    [notice, setNotice] = useState("");
  useEffect(() => {
    let live = true;
    setLoading(true);
    setError("");
    api
      .technicalFailures(offset)
      .then((result) => {
        if (live) setPage(result);
      })
      .catch((cause) => {
        if (live) setError(String(cause));
      })
      .finally(() => {
        if (live) setLoading(false);
      });
    return () => {
      live = false;
    };
  }, [offset, refresh]);
  return (
    <section
      className="technical-failures"
      aria-label="Detalhes das falhas técnicas"
    >
      <div className="section-heading">
        <div>
          <h3>Arquivos com falhas técnicas</h3>
          <p>
            {page?.total ?? "…"} arquivos; um arquivo pode ter falha de preview
            e metadados.
          </p>
        </div>
        <div>
          <button
            onClick={() => {
              setOffset(0);
              setRefresh((value) => value + 1);
            }}
          >
            Atualizar lista
          </button>
          <button onClick={close}>Fechar lista</button>
        </div>
      </div>
      {notice && <p role="status">{notice}</p>}
      {loading ? (
        <p role="status">Carregando falhas…</p>
      ) : error ? (
        <p role="alert">{error}</p>
      ) : page?.items.length ? (
        <>
          {page.items.map((item) => (
            <details key={item.assetId} className="technical-failure">
              <summary>
                <strong>{item.filename}</strong>
                <span>
                  {[
                    item.previewError && "Preview",
                    item.metadataError && "Metadados",
                  ]
                    .filter(Boolean)
                    .join(" · ")}
                </span>
              </summary>
              <p className="failure-path">{item.path}</p>
              {[
                ["Preview", item.previewError, item.previewUpdatedAt],
                ["Metadados", item.metadataError, item.metadataUpdatedAt],
              ].map(
                ([stage, reason, date]) =>
                  reason && (
                    <div key={stage}>
                      <h4>{stage}</h4>
                      <p>{reason}</p>
                      <small>
                        Último registro:{" "}
                        {date ? formatDate(date) : "Não disponível"}
                      </small>
                      <p>{guidance(reason)}</p>
                    </div>
                  ),
              )}
              <button
                onClick={() =>
                  void api
                    .revealAsset(item.assetId)
                    .then((result) =>
                      setNotice(
                        result === "selected"
                          ? "Arquivo selecionado no Explorador."
                          : "Pasta aberta; não foi possível selecionar o arquivo.",
                      ),
                    )
                    .catch((cause) => setNotice(String(cause)))
                }
              >
                Mostrar arquivo no Explorador
              </button>
            </details>
          ))}
          <div className="failure-pagination">
            <button
              disabled={offset === 0}
              onClick={() => setOffset(Math.max(0, offset - 50))}
            >
              Página anterior
            </button>
            <span>
              {offset + 1}–{offset + page.items.length} de {page.total}
            </span>
            <button
              disabled={page.nextOffset == null}
              onClick={() => setOffset(page.nextOffset!)}
            >
              Próxima página
            </button>
          </div>
        </>
      ) : (
        <p>Nenhuma falha técnica nesta consulta.</p>
      )}
    </section>
  );
}
