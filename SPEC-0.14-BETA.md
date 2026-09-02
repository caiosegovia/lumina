# Lumina 0.14 beta — revisão, descoberta e limpeza segura

## Objetivo

Transformar o catálogo em uma biblioteca viva: sincronizar fontes existentes, revelar pendências, revisar mídias rapidamente, explicar duplicatas e organizar o acervo sem alterar os originais.

## Escopo obrigatório

### Sincronização incremental

- Reanalisar uma ou todas as fontes sem criar uma nova fonte lógica.
- Reutilizar tamanho, modificação e hash persistidos quando a evidência for suficiente.
- Classificar ocorrências como presentes, ausentes, alteradas ou novas sem apagar histórico.
- Persistir progresso e permitir retomada após reinício.
- Nunca escrever na fonte durante inventário ou reconciliação.

### Central de revisão

- Unificar revisão posterior, data suspeita, preview ausente, metadado incompleto, falha técnica, duplicata pendente e proteção pendente.
- Oferecer filtros, contagens e ações rápidas com avanço automático.
- Manter uma trilha reversível de alterações de catálogo.

### Duplicatas

- Comparar visualmente ocorrências idênticas, proteção, origem, caminho e espaço potencial.
- Persistir decisão por ocorrência e por grupo.
- Gerar plano e relatório de limpeza sem remover fisicamente arquivos nesta beta.
- Bloquear candidatas sem réplica verificada.

### Visualizador

- Tela cheia, filmstrip, anterior/próxima, atalhos, zoom e pan.
- Entrega progressiva: miniatura imediata e preview de até 2560 px sob demanda, cacheado por versão e hash.
- Originais grandes não podem ser acumulados na memória do processo de interface.
- Reprodução de vídeo e comparação lado a lado quando suportado.
- Organização pessoal e metadados sincronizados durante a navegação.
- EXIF real é extraído sob demanda, persistido e apresentado com estados de leitura, ausência legítima e falha.

### Organização inteligente

- CRUD completo de visões salvas e álbuns inteligentes.
- Regras combináveis por período, mídia, origem, câmera, formato, tag, avaliação, revisão e proteção.
- CRUD de tags e ações em lote sem modificar os arquivos.

### Saúde operacional

- Estado agregado de catálogo, fontes, miniaturas, réplica, filas e ferramentas externas.
- Progresso uniforme, histórico, reparos em segundo plano e falhas acionáveis.
- Diagnóstico exportável sem caminhos, nomes, hashes, GPS ou conteúdo pessoal.
- Sessões anormais, panics e falhas do frontend são registradas localmente com rotação e sanitização.

### Sistema de interação

- Botões, campos, foco, loading, disabled e ações destrutivas seguem variantes visuais compartilhadas.
- Pills e agregadores são informativos e acionáveis; filtros ativos permanecem explícitos.
- Estados vazios distinguem ausência de dados, análise ainda não executada e falha.

## Fronteiras arquiteturais

- `catalog`: schema e migrações transacionais; nenhum caso de uso de UI.
- `sync`: reconciliação somente leitura das fontes e persistência de evidências.
- `review`: consultas e decisões reversíveis de revisão.
- `duplicates`: agrupamento, elegibilidade e plano de limpeza; nenhuma remoção.
- `health`: agregação de sinais e recomendações, sem executar reparos implicitamente.
- `jobs`: execução durável, cancelamento, retomada e eventos de progresso.
- comandos Tauri apenas validam DTOs e delegam casos de uso.

## Gates da beta

- Migração de catálogo anterior testada com snapshot e rollback por falha.
- Testes unitários, integração, frontend e E2E empacotado aprovados.
- `cargo fmt`, Clippy com `-D warnings`, build TypeScript e auditorias aprovados.
- Restart, volume offline, falta de espaço e ferramenta ausente cobertos.
- Benchmark reproduzível em catálogo grande e homologação funcional em `D:\Galeria Caio`.
- EXE, MSI e instalador beta gerados com hashes publicados.

## Fora desta beta

- Exclusão física, lixeira de fontes, reconhecimento facial, upload de conteúdo e acesso remoto.
