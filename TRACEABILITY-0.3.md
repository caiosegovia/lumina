# Rastreabilidade 0.3

| Requisito | Implementação | Evidência |
|---|---|---|
| Filtros combinados | `gallery.rs`, `Gallery.tsx` | `gallery::tests::filters_and_stats`, `Gallery.test.tsx` |
| Cursor estável | `gallery.rs` | `gallery::tests::cursor_no_overlap` |
| Entrada SQL vinculada | `gallery.rs` | `gallery::tests::input_is_bound` |
| 100 mil ativos | `catalog.rs` | `catalog_handles_one_hundred_thousand_assets` |
| Virtualização | `@tanstack/react-virtual`, `Gallery.tsx` | build e testes React |
| Big numbers/anos | `GallerySummary`, `Gallery.tsx` | `filters_and_stats`, teste React |
| Preview real | `AssetPreview`, `MediaThumb` | teste “usa a miniatura real também no preview” |
| Auditoria/reparo | `media::audit_thumbnails` | `audit_repairs_missing_and_corrupt_thumbnails_for_every_asset` |
| Segurança do cache | `thumbnail_data` | `internal_thumbnail_reader_rejects_cataloged_path_outside_cache` |
| Dependências | `package-lock.json` | `npm audit` sem vulnerabilidades |
