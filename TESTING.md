# Matriz de testes do Lumina 0.2.0

O comando oficial é:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\verify-0.2.ps1
```

Ele encerra imediatamente na primeira falha e executa formatação Rust, testes Rust, testes React/TypeScript e build web de produção.

## Cobertura automatizada

| Área | Evidência executável |
| --- | --- |
| Jobs e recuperação | estados persistentes, transições válidas, cancelamento cooperativo, retomada idempotente, descarte restrito aos temporários do job e bloqueio de duas instâncias |
| Armazenamento | SHA-256, espaço insuficiente, colisões, temporário verificado, promoção atômica e preservação byte a byte da fonte |
| Processos | único runner, ferramenta empacotada, segredo sanitizado, dependência ausente, timeout, cancelamento em execução, stdout de 1 MiB sem deadlock e concorrência máxima de dois |
| Validação | extensão falsa, imagem decodificada, vídeo real com FFprobe/frame, HEIC real, DNG RAW real e corrupção em revisão |
| Miniaturas | cache versionado, orientação EXIF, frame de vídeo, HEIC, prévia RAW, limpeza e reconstrução |
| Pipeline | deduplicação, reimportação, colisões, backup/manifesto, catálogo copiado e conferência integral |
| Relatórios | paginação, JSONL acima de 200 eventos e CSV escapado |
| Interface | progresso geral/etapa, acervo/backup separados, pausa, retomada, cancelamento, navegação, recuperação, recibo, retry, cache e placeholders |
| Carga | catálogo com 100 mil ativos, lote de 2 mil arquivos, vídeo válido de 64 MiB e consultas indexadas |

## Gates de distribuição

```powershell
npm.cmd run tauri -- build
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-0.2.ps1
```

O portátil inclui FFmpeg, FFprobe, ExifTool, licenças, manifesto e somas SHA-256.

## Aceite manual seguro

Use mídias descartáveis ou cópias controladas, nunca o único exemplar do acervo.

1. Extraia o ZIP e abra `Lumina.exe`; nenhuma janela de terminal deve aparecer.
2. Crie mestre e backup em pastas temporárias diferentes.
3. Analise JPG, HEIC, RAW e vídeo; navegue durante a análise.
4. Consolide, pause, retome e confira as duas barras e estados separados.
5. Feche durante outro trabalho; reabra e confirme Retomar/Descartar.
6. Confira miniaturas, recibo, JSONL/CSV, duplicatas e réplica.
7. Desconecte uma fonte sem cópia em curso e confirme que aparece offline, não apagada.

## Limites intencionais da 0.2

Reconhecimento facial/IA, similaridade visual, Google Takeout, mapa completo, compartilhamento, servidor Linux, exclusão automática, editor e API direta do Google continuam fora do pacote. O Lumina confirma a réplica local do Google Drive, não o upload remoto.
