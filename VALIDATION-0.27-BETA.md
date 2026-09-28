# Roteiro de homologação — 0.27.0-beta.1

Status: candidata. A aprovação automatizada local não substitui a homologação no catálogo de aproximadamente 9.300 arquivos.

## 1. Upgrade e regressão crítica

1. Feche a versão anterior, instale a 0.27 e abra a biblioteca de homologação. Confirme os mesmos caminhos de acervo/réplica e os mesmos totais.
2. Importe uma pasta pequena com foto, vídeo e uma duplicata. Conclua consolidação e proteção. Feche e reabra; o job deve estar concluído, não preso.
3. Abra arquivos no Explorer, confira data de captura, pasta física, favoritos e tags já existentes.
4. Navegue e use foto/vídeo, zoom, tela cheia e redimensionamento por dez minutos. Reprovar se houver lentidão progressiva, fechamento ou controles sumindo.

## 2. Seleção e produtividade

1. Selecione uma mídia e use `Shift` na quinta: o intervalo deve ficar destacado. Alterne grade/lista e confirme a seleção.
2. Com várias selecionadas, teste `F`, `Shift+F`, `R`, `Shift+R` e notas `0–5`. Confira o resultado após recarregar a galeria.
3. Teste tag, álbum, data e nome de lugar. Após cada ação use **Desfazer**: todo o lote deve voltar junto, não apenas o último arquivo.
4. Salve filtros como visão e álbum inteligente. Importe/edite uma mídia que passe a atender o filtro e reabra a visão; o resultado deve ser dinâmico.

## 3. Curadoria retomável

1. Aplique um filtro de ano ou tipo, abra **Curadoria**, crie uma sessão e confirme total/progresso.
2. Selecione algumas mídias e marque **Revisadas**; selecione outras e **Pular nesta sessão**.
3. Saia da sessão, feche o app, reabra **Curadoria** e retome. Os itens decididos não devem reaparecer e o progresso deve permanecer.
4. Termine a fila: estado **Concluída**. Exclua a sessão e confirme que favoritos/notas/tags das mídias permanecem.

## 4. Comparação

1. Selecione duas, três e quatro mídias e abra **Comparar**. Todas devem aparecer; diferenças de dimensão, câmera, lente, captura, tamanho, data ou origens ficam destacadas.
2. Com zoom sincronizado, amplie e altere enquadramento horizontal/vertical. Todas devem acompanhar sem barra de rolagem inferior.
3. Desmarque sincronização e amplie apenas uma mídia. Vídeos continuam reproduzíveis.
4. Escolha uma candidata: ela recebe favorito/nota, alternativas ficam em revisão, nenhuma mídia é excluída. Feche e use **Desfazer** na galeria.

## 5. Descobrir, bursts e lugares

1. Abra Descobrir duas vezes: a segunda abertura deve usar o snapshot sem pausa perceptível. Importe ou altere uma data e confirme atualização posterior do agrupamento.
2. Confira cobertura e abra um mês; a galeria deve conter exatamente o período.
3. Em um burst, **Ajustar grupo**, desmarque uma associação errada, salve e reabra. O arquivo sai apenas daquele agrupamento. Selecione todos novamente para restaurar.
4. Abra o grupo completo e confirme que não é uma busca textual aproximada.
5. Na galeria, selecione fotos GPS de lugares diferentes e aplique um nome. Somente as selecionadas recebem o nome manual; use Desfazer. O original/EXIF não muda.

## 6. Revisão e atividade

1. Abra **Falhas técnicas**, filtre por Preview, Metadados e nome/caminho. Paginação e total devem seguir o filtro.
2. Erro transitório de timeout/busy oferece nova tentativa de preview. Formato incompatível/corrompido não oferece retry enganoso.
3. Na Atividade, trabalhos ativos, aguardando ação, manutenção parcial e histórico devem permanecer separados. Proteção e importação continuam atualizando progresso.

## 7. Soak

Use por 45–60 minutos no acervo de homologação: navegação rápida, curadoria, comparação, Descobrir e um job em segundo plano. Ao final exporte o diagnóstico. Retorno: `item | aprovado/reprovado | ação/arquivo | horário local | evidência`.

## Evidências locais exigidas antes da tag

- Frontend, TypeScript/Vite, Rust, formatação e Clippy sem falhas/avisos.
- Browser Edge nos três tamanhos/DPI sintéticos e temas claro/escuro.
- Executável distribuído em perfil isolado: setup, importação, deduplicação, proteção, índice, curadoria retomável, undo de lote, decisão de comparação, foto/vídeo e encerramento limpo.
- Instaladores NSIS e MSI gerados com recursos completos.

## Evidências executadas em 28/09/2026

- `npm test`: 58 testes aprovados em 9 arquivos.
- `npm run build`: TypeScript e Vite aprovados.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`: aprovados.
- `cargo test --lib -- --test-threads=1`: 147 aprovados, 2 ignorados e nenhuma falha.
- `scripts/verify-0.27.ps1 -SkipDebugResourceCopy`: aprovado com perfil isolado.
- Smoke do viewer no Edge: aprovado em 1400×900/DPR 1, 1100×700/DPR 1,25 e 1000×700/DPR 1,5.
- Smoke do executável extraído do MSI: setup, análise de foto/vídeo/duplicata, consolidação, réplica verificada, curadoria retomável, comparação, undo atômico, reprodução, zoom e encerramento limpo aprovados.
- O smoke alterou somente dados sintéticos em `artifacts/0.27/desktop-1790614474303`; o arquivo-fonte permaneceu byte a byte inalterado.
- Instaladores produzidos: `Lumina_0.27.0-1_x64-setup.exe` e `Lumina_0.27.0-1_x64_en-US.msi`.
