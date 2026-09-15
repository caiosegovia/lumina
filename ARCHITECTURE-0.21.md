# Arquitetura — Lumina 0.21

## Tempo

```mermaid
flowchart LR
  EXIF[DateTimeOriginal] --> OFFSET{Offset informado?}
  OFFSET -->|sim| WALL[Relógio da câmera + proveniência]
  OFFSET -->|não| FLOAT[Hora local flutuante]
  FILE[Data do filesystem] --> INSTANT[Instante absoluto]
  WALL --> CAT[(captured_at / date_source)]
  FLOAT --> CAT
  INSTANT --> CAT
  CAT --> UI[Formatação coerente na Galeria]
```

Datas de câmera não recebem UTC inventado. A migração v17 é transacional e corrige apenas fontes históricas afetadas. Jobs e telemetria permanecem em UTC.

## Lugares

O resolvedor consulta em lote a base `Geolocation.dat` já embarcada no ExifTool. Se a mídia estiver indisponível ou não houver resposta, usa o gazetteer interno com limite de 45 km; fora desse limite mantém região aproximada. Cache automático e override manual continuam separados.

## Bursts

```mermaid
flowchart LR
  MEDIA[Foto/RAW ordenada] --> GAP[Intervalo temporal]
  GAP --> CAMERA[Mesmo equipamento]
  CAMERA --> FAST{até 2 s?}
  FAST -->|sim| BURST[Burst]
  FAST -->|não, até 8 s| VIS[Distância perceptual]
  VIS -->|coerente| BURST
  VIS -->|divergente| SPLIT[Novo evento]
  BURST --> BEST[Melhor candidata explicada]
  BEST --> CURATE[Favorita + revisão reversível]
```

O algoritmo opera sobre fingerprints locais versionados. A interface apenas solicita decisões; persistência e integridade permanecem no backend/catalogo.

O desenho consolidado está em `docs/design/time-places-bursts-0.21.svg`.
