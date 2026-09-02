# Lumina 0.15.0-beta.2

Candidata consolidada do ciclo 0.15: workspace, produtividade, inspeção, comparação e processamento invisível no mesmo pacote.

## Produtividade da galeria

- Ordenação no backend por captura, nome ou tamanho, ascendente ou descendente.
- Cursor coerente com cada ordenação; nenhuma ordenação limitada à página carregada.
- Densidades confortável e compacta específicas da lista, com preferência persistente.
- Seleção por intervalo usando Shift e seleção de todas as mídias carregadas.
- Ações em lote com quantidade explícita e acesso a desfazer.
- Grade e lista usam a mesma consulta, seleção, filtros e item inspecionado.

## Inspeção e comparação

- Metadados em seções recolhíveis de captura, arquivo/mídia, localizações e catálogo.
- Preferências das seções persistentes.
- Cópia explícita do SHA-256 e abertura do arquivo no Explorador.
- Comparação de duas mídias a partir da seleção da galeria.
- Preview HD ou vídeo, zoom compartilhado e dados essenciais lado a lado.
- A comparação é informativa: não marca duplicidade e não remove arquivos.

## Processamento invisível

- Mídias no viewport solicitam miniaturas com prioridade interativa.
- Próximos itens recebem prefetch limitado e de prioridade menor.
- Solicitações repetidas são consolidadas antes de chegar à fila persistente.
- Uma solicitação posterior pode elevar a prioridade de um trabalho já pendente.
- Retomada de manutenção continua automática e sem modal.
- Cache e derivados continuam reconstruíveis; originais permanecem somente leitura.

## Refinamento da experiência

- Visão geral orientada a memórias, com primeira e última foto e vídeo.
- Capacidade, proteção, acervo principal e réplica local consolidados em uma única superfície.
- Composição simplificada para fotos e vídeos; ritmo e linha do tempo preservados.
- Inventário técnico separa formatos, equipamentos normalizados e codecs.
- Insights usam cards acionáveis e desempenho compara até cinco jobs por etapa.
- Seleção da galeria ganhou preenchimento, contorno e identificação textual.
- Inspetor sem faixa rolável inferior e com metadados focados no uso real.

## Roteiro de homologação

1. Ordene por cada opção, role até carregar outra página e verifique continuidade sem repetição.
2. Alterne grade/lista e as duas densidades; confirme persistência após reiniciar.
3. Selecione uma mídia, use Shift em outra e confirme o intervalo.
4. Selecione duas mídias, abra Comparar e teste zoom, foto/vídeo e fechamento com Escape.
5. Abra o inspetor, recolha seções, navegue e reabra o aplicativo para conferir preferências; confirme que não há faixa rolável sob o preview.
6. Copie o hash e abra a localização pelo botão explícito.
7. Role rapidamente por uma galeria extensa e confirme que miniaturas visíveis chegam antes das seguintes.
8. Feche o app com miniaturas pendentes, reabra e confirme retomada silenciosa.
9. Repita favoritos, tags, álbuns, duplicatas, reparo e preview HD para regressão.
10. Confirme que nenhuma fonte ou original foi modificado.
11. Confira primeira/última foto e vídeo, composição, nomes de equipamentos e gráficos de capacidade.
12. Depois de ao menos dois jobs medidos, confira barras e tempos no comparativo de desempenho.

## Integridade dos downloads

- Portátil: `93733a4463c58709cc8f99cfb3b1e6dd7faa3a210633544f598740ae8d40f530`
- MSI: `94ec8a6075ed95cdabae48d49c927b51cafaf98aaee3f2cf8da3a3de249e29a0`
- Instalador EXE: `1ee1c40cfdd9a7dce557761d8ef4e1fbb2b187037cef39a0a49de961a3b4ddae`
