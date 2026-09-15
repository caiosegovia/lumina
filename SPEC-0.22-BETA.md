# Especificação — Lumina 0.22.0-beta.1

## Objetivo

Transformar GPS em contexto confiável e acelerar a curadoria sem alterar arquivos originais.

## Escopo

1. Localizações v3 leem cidade, estado, país, bairro/sublocalização, altitude e precisão presentes em EXIF, XMP, IPTC e QuickTime.
2. Dados textuais do arquivo têm precedência sobre o geocoder offline; correções humanas continuam soberanas.
3. Células geográficas passam de cerca de 5 km para aproximadamente 100 m e o cache automático torna-se atualizável e versionado.
4. A migração v18 preserva datas, tags, favoritos, nomes manuais e cria detalhes geográficos e preferências.
5. O inspetor explica nome, hierarquia, origem, precisão e altitude e permite copiar coordenadas.
6. “Mostrar arquivo no Explorador” seleciona o arquivo com argumento separado e informa sucesso ou erro.
7. Comparação aceita de duas a quatro mídias com zoom sincronizado e metadados lado a lado.
8. Perfis Economia, Equilibrado e Desempenho controlam o limite global de I/O.
9. Regras Equilibrada, Priorizar qualidade e Revisar todas são persistentes e aplicadas à curadoria de bursts.

## Innegociáveis

- Operação local e privada; nenhuma consulta de GPS à internet.
- Nenhuma gravação em EXIF/XMP/IPTC nem exclusão automática.
- UI responsiva, ações reversíveis e estados explícitos.
- Migração transacional e repetível a partir da 0.21.
