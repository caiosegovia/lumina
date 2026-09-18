# Revisao visual 0.23 beta 3

## Objetivo

Restaurar a identidade aprovada do Lumina e aplicar uma linguagem consistente em todas as jornadas, sem alterar os contratos funcionais e de resiliencia homologados na 0.22.2.

## Fundacao

- DM Sans Variable local para interface e Manrope Variable local para titulos e numeros de destaque.
- Nenhuma fonte ou folha de estilo depende de rede.
- Tokens semanticos para tela, superficie, texto, borda, destaque, sucesso, atencao, erro, overlay e palco de midia.
- Controles compartilham altura, raio, foco, hover, estado pressionado e estado desabilitado.
- Temas Claro, Escuro e Seguir Windows no cabecalho; a preferencia persiste localmente.
- Movimento reduzido respeita a configuracao de acessibilidade do sistema.

## Superficies revisadas

| Area | Tratamento |
|---|---|
| Navegacao | contraste, selecao, hierarquia, badge e botao de importacao |
| Visao geral | cards, numeros, paineis, graficos, capacidade, protecao e insights |
| Galeria | comando fixo, pills, grade, lista, selecao e estados de midia |
| Inspector | preview neutro, metadados, navegacao, filmstrip e tela cheia |
| Comparacao | overlay, paineis, metadados e palco de foto/video |
| Atividade | abas, jobs, progresso, estados de atencao e processamento invisivel |
| Duplicatas | lista compacta, filtros, selecao, expansao e acoes em lote |
| Descobrir | grupos, indice, preferencias, lugares e estados vazios |
| Fontes e protecao | disponibilidade, alertas, capacidade e acoes |
| Modais e onboarding | superficies, campos, foco, overlay e mensagens |

## Roteiro visual

1. Confirme no cabecalho os modos Claro, Seguir Windows e Escuro.
2. Alterne entre os tres modos em cada secao; a navegacao e o trabalho atual nao podem reiniciar.
3. Feche e reabra no modo Escuro e confirme a persistencia; repita com Seguir Windows.
4. Compare todas as secoes nos dois temas e procure superficies claras ou textos sem contraste.
5. Verifique DM Sans nos textos, Manrope nos titulos e ausencia de troca de fonte depois do carregamento.
6. Teste foco pelo teclado, hover, selecionado, desabilitado, alertas, modais e menus.
7. Repita em escala de 100%, 125% e 150% do Windows e com a janela reduzida.
8. Confirme que foto e video preservam suas cores e que preview, zoom e tela cheia continuam sincronizados.

## Regras de aceitacao

1. A familia calculada do texto de interface deve iniciar por `DM Sans Variable`.
2. Titulos devem usar `Manrope Variable`.
3. Alternar tema nao pode recarregar a tela, perder selecao ou interromper jobs.
4. Claro ou Escuro deve permanecer escolhido depois de reiniciar.
5. Seguir Windows deve reagir a mudanca do sistema durante a execucao.
6. Nenhuma tela pode exibir superficie clara ilegivel no tema escuro.
7. Foco de teclado deve permanecer visivel nos dois temas.
8. Fotos e videos nao podem receber filtros ou alteracao de cor pelo tema.
9. A interface deve permanecer utilizavel a 100%, 125% e 150% de escala.
10. A revisao nao pode mudar catalogo, originais, jobs ou dados editoriais.

## Fora do escopo

Esta revisao nao muda pipeline de importacao, banco, metadados, duplicatas, localizacao ou protecao. Defeitos funcionais encontrados durante a homologacao devem ser registrados separadamente da avaliacao visual.
