# Lumina 0.15.0-beta.1

Primeira candidata do novo workspace da galeria.

Consulte [DESIGN-0.15-GALLERY.md](DESIGN-0.15-GALLERY.md) para comparar a implementação com a anatomia e os estados esperados.

## Entregas desta candidata

- Segmentação e comandos da galeria em uma região fixa durante a navegação.
- Inspetor de mídia integrado ao workspace, sem cobrir grade ou lista em telas amplas.
- Lista redesenhada com colunas de mídia, captura, arquivo, origem e proteção.
- Estados visuais modernos para hover, seleção, proteção e informações pessoais.
- Adaptação progressiva do workspace para larguras menores.
- Sequência do visualizador explicada, recolhível e persistente.
- Preservação das funcionalidades homologadas na 0.14.0-beta.4.

## Roteiro focal

1. Alterne entre grade e lista e confirme que filtros, agrupamento e seleção continuam ativos.
2. Role uma galeria extensa e confirme que a segmentação permanece acessível.
3. Abra uma mídia e continue navegando pela galeria com o inspetor visível.
4. Navegue entre fotos e vídeos pelo inspetor e confirme preview, metadados e posição.
5. Recolha a seção Sequência, feche e reabra o inspetor e confirme a preferência.
6. Redimensione a janela e confirme a adaptação sem conteúdo ilegível ou inacessível.
7. Repita favoritos, tags, álbuns, reparo, duplicatas e reinício para detectar regressões.
