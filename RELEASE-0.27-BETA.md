# Lumina 0.27.0-beta.2 — produtividade e curadoria

Versão candidata para homologação. O pacote é aditivo: não muda a importação, a proteção, a organização física por data nem os arquivos originais.

## Entregas

- Seleção contínua com `Shift`, seleção das mídias carregadas e atalhos de catálogo: `F` favorita, `Shift+F` remove favorita, `R` revisa depois, `Shift+R` conclui e `0–5` atribui nota.
- Ações em lote para tag, álbum, data, lugar, favorito, nota e revisão. Uma única ação de lote gera uma única unidade de undo, restaurada de forma transacional.
- Sessões de curadoria materializadas no catálogo: nome, filtros, ordenação, progresso, itens revisados/pulados, retomada depois de fechar o aplicativo e exclusão da sessão sem perder decisões feitas nas mídias.
- Visões salvas e álbuns inteligentes continuam dinâmicos; o diálogo nativo antigo foi substituído por um fluxo visual próprio.
- Comparação de duas a quatro mídias com preview limitado, zoom de 100–400%, modo sincronizado ou independente, ajuste de enquadramento, diferenças técnicas destacadas e escolha da melhor candidata. A escolha favorita/avalia a vencedora, envia alternativas para revisão e pode ser desfeita.
- Bursts abrem pelo conjunto exato de IDs, explicam o critério e permitem retirar associações erradas apenas no catálogo. Nenhum arquivo é apagado ou movido.
- Descobrir mostra cobertura do índice e acesso por mês. O overview passa a possuir snapshot persistente invalidado por triggers quando assets, índice visual, localização ou ajustes mudam; visitas repetidas não refazem os agrupamentos sem necessidade.
- Lugares distinguem origem embutida, estimada e manual. A galeria permite aplicar nome somente às mídias selecionadas; o override por asset tem precedência sobre o nome da célula e é reversível.
- Falhas técnicas podem ser filtradas por etapa, nome ou caminho. Nova tentativa de preview só aparece para erros transitórios; formatos incompatíveis não recebem promessa de reparação.
- O ciclo de vida dos jobs agora fecha o lease e as conexões do catálogo antes de publicar o worker como ocioso, eliminando uma janela de corrida observada no CI do Windows.
- A passagem análise → consolidação é enfileirada no catálogo quando um worker ainda está finalizando. Promoções atômicas de previews e snapshots aguardam, de forma limitada, bloqueios transitórios do Windows sem transformar falhas permanentes em loop.

## Preservado

Setup com acervo e réplica, importação durável, proteção verificada, datas de captura e estrutura física, Explorer, visualizador de foto/vídeo, temas, tipografia, duplicatas sem exclusão automática e limites de recursos da baseline resiliente.

## Instalação

Use o instalador NSIS da tag `v0.27.0-beta.2`; o MSI é alternativo. A versão do bundle Tauri é `0.27.0-2`. Feche a versão anterior antes de atualizar. Para uma experiência limpa, remova apenas o perfil de teste e nunca o catálogo definitivo sem cópia independente.

Checksums SHA-256:

- `Lumina_0.27.0-2_x64-setup.exe`: `519F7027B55B9BE4D705A160E090D6AF3762A83763ACF5330F032BC522BDFF06`
- `Lumina_0.27.0-2_x64_en-US.msi`: `F3B54AEA42C0FB4C89A8089C767B2D4FBBA87980AB4721B36E29907E79BB934A`

Roteiro: [VALIDATION-0.27-BETA.md](VALIDATION-0.27-BETA.md). Arquitetura: [ARCHITECTURE-0.27.md](ARCHITECTURE-0.27.md).
