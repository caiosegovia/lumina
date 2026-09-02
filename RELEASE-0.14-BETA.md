# Lumina 0.14.0-beta.4

O pacote MSI usa a versão técnica `0.14.0-4`, equivalente à beta.4. Esta é a candidata funcional para homologação no dispositivo oficial.

## Destaques desta candidata

- Visualizador estável: fotos deixam de carregar originais completos a cada navegação.
- Preview progressivo: miniatura imediata e representação JPEG de até 2560 px gerada sob demanda em processo isolado.
- Cache de visualização versionado por hash, reutilizável e limitado a 2 GiB.
- Metadados EXIF reais extraídos ao abrir a foto e persistidos no catálogo.
- Estados explícitos durante leitura de metadados e geração de preview HD.
- Reprodução de vídeo preservada com streaming local e suporte a intervalos de bytes.
- Galeria com agregadores, filtros rápidos e pills de mídia, formato, favorito, tags e proteção.
- Sistema visual unificado para botões, campos, estados, foco, carregamento e ações destrutivas.
- Saúde da biblioteca reorganizada por prioridade; versões de ferramentas ficam em detalhes técnicos.
- Reparo de miniaturas com total, processadas, recuperadas e falhas em tempo real.
- Duplicatas distinguem catálogo não analisado, análise concluída sem cópias e grupos encontrados.
- Registro local rotativo de sessão anormal, panics e erros sanitizados do frontend/mídia.

## Funcionalidades preservadas

- Sincronização incremental e retomável, sem escrever nas fontes.
- Central de Revisão, ações rápidas e desfazer última edição.
- Decisões persistentes de duplicatas e plano de limpeza sem exclusão física.
- CRUD de visões, álbuns inteligentes, álbuns manuais e tags.
- Favoritos, avaliações, revisão posterior e ações em lote persistentes.
- Proteção com réplica verificada e diagnóstico seguro.

## Segurança e limites

- Nenhuma exclusão física está habilitada.
- Fontes continuam somente leitura.
- O preview HD é derivado e reconstruível; o original não é alterado.
- Logs locais não registram nomes de mídias, caminhos, hashes ou GPS.
- Similaridade visual, reconhecimento facial, nuvem e acesso remoto permanecem fora deste ciclo.

## Roteiro de homologação

1. Navegue rapidamente por fotos, use zoom e tela cheia e confirme estabilidade e nitidez do selo **Prévia HD**.
2. Abra fotos JPEG/RAW e confirme câmera, lente, ISO, abertura, exposição e distância focal quando presentes no arquivo.
3. Use as pills da galeria e confirme que as contagens e resultados mudam juntos.
4. Abra **Duplicatas** e confira o estado da análise, fontes conectadas e grupos exatos.
5. Em **Proteção**, confira a nova Saúde e execute o reparo observando o progresso numérico.
6. Reinicie o aplicativo e confirme persistência de favoritos, tags, álbuns e decisões.
7. Exporte o diagnóstico e confirme que nenhuma fonte ou mídia foi modificada.
