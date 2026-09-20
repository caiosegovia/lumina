# Roteiro de homologação — 0.24.2 beta 1

## Atualização e processamento

1. Instale sobre a 0.24.1 e abra a biblioteca de homologação.
2. Confirme que as 5.548 miniaturas saudáveis continuam disponíveis sem reconstrução geral.
3. Aguarde o reprocessamento seletivo dos 107 DNG, 1 CR2 e 1 MP4 registrados no último diagnóstico.
4. Em Atividades, o processamento de miniaturas deve concluir; arquivos realmente incompatíveis aparecem como limitações, não como job preso.
5. Exporte um novo diagnóstico e compare `thumbnailFailures` com a linha de base de 109.

## Galeria e zoom

6. Abra foto JPEG e RAW. Confirme “Prévia HD” e navegue entre imagens sem reter o zoom anterior.
7. Use roda do mouse sem Ctrl sobre quatro pontos diferentes; o detalhe sob o cursor deve permanecer ancorado.
8. Arraste ampliado até as bordas: a imagem não pode ser perdida fora da área visível.
9. Teste Ajustar, 100%, Preencher, +, -, 0, duplo clique e tela cheia.
10. Confirme fluidez durante navegação prolongada e ausência de crescimento contínuo de memória após fechar o inspetor.

## Proteção e Explorer

11. Force ou observe falta de espaço: necessidade e espaço livre devem aparecer em GB/TB, nunca em bytes crus.
12. Clique em “Mostrar arquivo no Explorador”. Aceite principal: arquivo destacado. Se o Windows impedir a seleção, o app deve abrir a pasta correta e dizer explicitamente que foi fallback.
13. Execute proteção e verificação; confirme avanço, conclusão e retomada após reinício.

Aceite: zero crash; catálogo íntegro; originais inalterados; job interno não preso; redução explicável das 109 falhas; zoom previsível; mensagens de capacidade legíveis.
