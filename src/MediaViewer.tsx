import { useCallback, useEffect, useRef, useState } from "react";
import {
  Images,
  LoaderCircle,
  Maximize2,
  Minimize2,
  Pause,
  Play,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import { api } from "./api";
import type { MediaAsset } from "./types";
import {
  actualSizeScale,
  clampTransform,
  fillScale,
  fittedSize,
  zoomAt,
  type ViewerTransform,
} from "./viewerTransform";

export default function MediaViewer({
  asset,
  fullscreen,
  toggleFullscreen,
}: {
  asset: MediaAsset;
  fullscreen: boolean;
  toggleFullscreen: () => void;
}) {
  const stage = useRef<HTMLDivElement>(null),
    video = useRef<HTMLVideoElement>(null);
  const [src, setSrc] = useState(""),
    [thumbnail, setThumbnail] = useState(""),
    [quality, setQuality] = useState("loading"),
    [error, setError] = useState("");
  const [natural, setNatural] = useState({
    width: asset.width || 0,
    height: asset.height || 0,
  });
  const [viewport, setViewport] = useState({ width: 0, height: 0 });
  const [transform, setTransform] = useState<ViewerTransform>({
    scale: 1,
    x: 0,
    y: 0,
  });
  const [mode, setMode] = useState<"fit" | "actual" | "fill" | "free">("fit");
  const [playing, setPlaying] = useState(false),
    [time, setTime] = useState(0),
    [duration, setDuration] = useState(0);
  const current = useRef(transform),
    frame = useRef<number>(),
    drag = useRef<{ x: number; y: number; start: ViewerTransform }>();
  const commit = useCallback((next: ViewerTransform) => {
    current.current = next;
    if (frame.current !== undefined) return;
    frame.current = requestAnimationFrame(() => {
      frame.current = undefined;
      setTransform(current.current);
    });
  }, []);
  const fit = useCallback(() => {
    setMode("fit");
    commit({ scale: 1, x: 0, y: 0 });
  }, [commit]);
  const zoom = useCallback(
    (scale: number, anchor = { x: 0, y: 0 }) => {
      setMode("free");
      commit(zoomAt(current.current, scale, anchor, viewport, natural));
    },
    [commit, viewport, natural],
  );

  useEffect(() => {
    let live = true,
      timer: ReturnType<typeof setTimeout>,
      attempts = 0;
    const load = async () => {
      try {
        const url =
          asset.mediaType === "video"
            ? await api.mediaUrl(asset.id)
            : await api.photoPreview(asset.id);
        if (live) {
          setSrc(url);
          setQuality(url ? "ready" : "loading");
        }
      } catch (cause) {
        if (!live) return;
        if (String(cause).includes("PREVIEW_BUSY") && attempts++ < 20) {
          timer = setTimeout(load, 300);
          return;
        }
        setQuality("error");
        setError(
          "Não foi possível preparar esta prévia. Consulte Revisão → Falhas técnicas.",
        );
      }
    };
    api
      .thumbnail(asset.id)
      .then((url) => {
        if (live) setThumbnail(url || "");
      })
      .catch(() => {});
    timer = setTimeout(load, 150);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [asset.id, asset.mediaType]);
  useEffect(() => {
    const element = stage.current;
    if (!element) return;
    const update = () => {
      const r = element.getBoundingClientRect();
      setViewport((old) =>
        old.width === r.width && old.height === r.height
          ? old
          : { width: r.width, height: r.height },
      );
    };
    update();
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(update);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    if (mode === "fit") commit({ scale: 1, x: 0, y: 0 });
    else if (mode === "actual")
      commit(
        zoomAt(
          current.current,
          actualSizeScale(viewport, natural),
          { x: 0, y: 0 },
          viewport,
          natural,
        ),
      );
    else if (mode === "fill")
      commit(
        zoomAt(
          current.current,
          fillScale(viewport, natural),
          { x: 0, y: 0 },
          viewport,
          natural,
        ),
      );
    else commit(clampTransform(current.current, viewport, natural));
  }, [viewport, natural, mode, commit]);
  useEffect(() => {
    const element = stage.current;
    if (!element) return;
    const wheel = (event: WheelEvent) => {
      event.preventDefault();
      const r = element.getBoundingClientRect();
      zoom(current.current.scale + (event.deltaY < 0 ? 0.25 : -0.25), {
        x: event.clientX - r.left - r.width / 2,
        y: event.clientY - r.top - r.height / 2,
      });
    };
    element.addEventListener("wheel", wheel, { passive: false });
    return () => element.removeEventListener("wheel", wheel);
  }, [zoom]);
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (
        (event.target as HTMLElement)?.closest(
          "input,textarea,select,[contenteditable=true]",
        )
      )
        return;
      if (["+", "=", "-", "0"].includes(event.key)) {
        event.preventDefault();
        if (event.key === "0") fit();
        else zoom(current.current.scale + (event.key === "-" ? -0.25 : 0.25));
      }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [zoom, fit]);
  useEffect(() => {
    const element = video.current;
    return () => {
      if (element) {
        element.pause();
        element.removeAttribute("src");
        element.load();
      }
    };
  }, [src]);
  useEffect(
    () => () => {
      if (frame.current !== undefined) {
        cancelAnimationFrame(frame.current);
        frame.current = undefined;
      }
    },
    [],
  );
  const size = fittedSize(viewport, natural);
  const style = {
    width: size.width || "100%",
    height: size.height || "100%",
    transform: `translate(-50%, -50%) translate(${transform.x}px, ${transform.y}px) scale(${transform.scale})`,
  };
  const endDrag = () => {
    drag.current = undefined;
  };
  const showImage = asset.mediaType !== "video" && (src || thumbnail);
  return (
    <section className="media-viewer" aria-label="Visualizador de mídia">
      <div
        ref={stage}
        className={`media-viewport preview-stage ${transform.scale > 1 ? "pannable" : ""}`}
        aria-label="Área da mídia"
        onDoubleClick={() => (current.current.scale === 1 ? zoom(2) : fit())}
        onPointerDown={(event) => {
          if (event.button !== 0 || current.current.scale <= 1) return;
          event.preventDefault();
          event.currentTarget.setPointerCapture?.(event.pointerId);
          drag.current = {
            x: event.clientX,
            y: event.clientY,
            start: current.current,
          };
        }}
        onPointerMove={(event) => {
          if (!drag.current) return;
          commit(
            clampTransform(
              {
                ...drag.current.start,
                x: drag.current.start.x + event.clientX - drag.current.x,
                y: drag.current.start.y + event.clientY - drag.current.y,
              },
              viewport,
              natural,
            ),
          );
        }}
        onPointerUp={(event) => {
          endDrag();
          if (event.currentTarget.hasPointerCapture?.(event.pointerId))
            event.currentTarget.releasePointerCapture(event.pointerId);
        }}
        onPointerCancel={endDrag}
      >
        {asset.mediaType === "video" && src ? (
          <video
            ref={video}
            src={src}
            className="viewer-media"
            style={style}
            preload="metadata"
            playsInline
            onLoadedMetadata={(event) => {
              const v = event.currentTarget;
              setNatural({ width: v.videoWidth, height: v.videoHeight });
              setDuration(Number.isFinite(v.duration) ? v.duration : 0);
            }}
            onPlay={() => setPlaying(true)}
            onPause={() => setPlaying(false)}
            onEnded={() => setPlaying(false)}
            onTimeUpdate={(event) => setTime(event.currentTarget.currentTime)}
            onError={() => setError("O vídeo não pôde ser reproduzido.")}
          />
        ) : showImage ? (
          <img
            className="viewer-media"
            src={src || thumbnail}
            alt={`Prévia de ${asset.filename}`}
            style={style}
            draggable={false}
            onLoad={(event) =>
              setNatural({
                width: event.currentTarget.naturalWidth,
                height: event.currentTarget.naturalHeight,
              })
            }
            onError={() => {
              setError("Não foi possível exibir esta prévia.");
              setQuality("error");
            }}
          />
        ) : (
          <Images className="viewer-placeholder" />
        )}
      </div>
      <div
        className="viewer-toolbar"
        role="toolbar"
        aria-label="Controles do visualizador"
      >
        <button
          aria-label="Ajustar imagem à tela"
          disabled={mode === "fit"}
          onClick={fit}
        >
          Ajustar
        </button>
        <button
          aria-label="Mostrar tamanho real"
          title="Um pixel da prévia por pixel da tela"
          onClick={() => setMode("actual")}
        >
          1:1
        </button>
        <button aria-label="Preencher área" onClick={() => setMode("fill")}>
          Preencher
        </button>
        <button
          aria-label="Diminuir zoom"
          disabled={transform.scale <= 1}
          onClick={() => zoom(current.current.scale - 0.25)}
        >
          <ZoomOut />
        </button>
        <span
          aria-label="Nível de zoom"
          title="Ampliação relativa ao enquadramento ajustado"
        >
          {Math.round(transform.scale * 100)}%
        </span>
        <button
          aria-label="Aumentar zoom"
          disabled={transform.scale >= 12}
          onClick={() => zoom(current.current.scale + 0.25)}
        >
          <ZoomIn />
        </button>
        <button
          aria-label={fullscreen ? "Sair da tela cheia" : "Abrir em tela cheia"}
          onClick={toggleFullscreen}
        >
          {fullscreen ? <Minimize2 /> : <Maximize2 />}
        </button>
      </div>
      {asset.mediaType === "video" && src && (
        <div
          className="viewer-playback"
          role="group"
          aria-label="Reprodução do vídeo"
        >
          <button
            aria-label={playing ? "Pausar vídeo" : "Reproduzir vídeo"}
            onClick={() => {
              const v = video.current;
              if (v) {
                if (v.paused)
                  void v
                    .play()
                    .catch(() =>
                      setError("Não foi possível iniciar a reprodução."),
                    );
                else v.pause();
              }
            }}
          >
            {playing ? <Pause /> : <Play />}
          </button>
          <input
            type="range"
            aria-label="Posição do vídeo"
            min={0}
            max={duration || 1}
            step={0.1}
            value={Math.min(time, duration || 1)}
            onChange={(event) => {
              if (video.current) {
                video.current.currentTime = Number(event.target.value);
                setTime(Number(event.target.value));
              }
            }}
          />
          <span>
            {Math.floor(time / 60)}:
            {String(Math.floor(time % 60)).padStart(2, "0")}
          </span>
          <input
            type="range"
            aria-label="Volume do vídeo"
            min={0}
            max={1}
            step={0.05}
            defaultValue={1}
            onChange={(event) => {
              if (video.current)
                video.current.volume = Number(event.target.value);
            }}
          />
        </div>
      )}
      <div className="viewer-message" aria-live="polite">
        {error ||
          (asset.mediaType !== "video" ? (
            quality === "ready" ? (
              "Prévia HD"
            ) : quality === "loading" ? (
              <>
                <LoaderCircle className="spin" /> Preparando alta qualidade
              </>
            ) : (
              ""
            )
          ) : (
            "Arraste para explorar o vídeo ampliado"
          ))}
      </div>
    </section>
  );
}
