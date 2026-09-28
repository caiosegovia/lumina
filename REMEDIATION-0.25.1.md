# Correção e prevenção — configuração e primeira importação

## Incidente

Na `0.25.0-beta.1`, uma configuração persistida era considerada pronta apenas por existir. Caminhos de acervo mestre ou réplica ausentes podiam deixar o onboarding fora da jornada e os jobs falhavam depois de entrar na fila com `os error 2`.

## Contratos corrigidos

- A inicialização possui três estados explícitos: `unconfigured`, `ready` e `needs_repair`.
- Acervo mestre, catálogo SQLite, réplica e independência entre os destinos são validados antes de iniciar watchdogs ou trabalhos em segundo plano.
- Uma configuração inválida abre a tela de reparo preenchida com os caminhos persistidos.
- O usuário escolhe mestre e réplica pelo seletor nativo; não existem mais letras de unidade presumidas.
- A identidade e a data de criação da biblioteca são preservadas durante o reparo.
- A mesma validação é executada antes da análise síncrona e antes da criação de um job assíncrono.
- Um caminho inválido gera erro por função (`acervo mestre`, `catálogo`, `réplica` ou `fonte`) e não cria trabalho condenado a falhar.

## Plano para impedir recorrência

### Gate automatizado por commit

1. Testes Rust de configuração ausente, catálogo ausente, destinos sobrepostos e configuração válida.
2. Teste de componente da tela `needs_repair`, incluindo seleção dos dois caminhos e bloqueio do dashboard.
3. Teste do preflight garantindo zero jobs criados quando a configuração não está pronta.
4. Build TypeScript/Vite, `cargo fmt`, `cargo clippy` e suíte Rust completos.

### Gate obrigatório de pacote

1. Instalação limpa em perfil isolado do Windows.
2. Criação de biblioteca com caminhos escolhidos pelo usuário.
3. Primeira importação real de um fixture com foto e vídeo.
4. Reinstalação preservando `%LOCALAPPDATA%` e configuração válida.
5. Reabertura com mestre ausente, réplica ausente e fonte removida.
6. Atualização a partir da última versão homologada com catálogo existente.
7. Confirmação de que um preflight recusado não deixa job, fonte ou contador órfão.

### Regra de publicação

Uma release não pode ser publicada com apenas um teste de abertura da janela. O relatório precisa registrar o commit, instalador, perfil usado, caminhos do fixture, resultado de cada cenário e diagnóstico exportado. Qualquer falha na instalação limpa, recuperação de configuração ou primeira importação reprova o candidato.

## Matriz de aceite da correção

| Cenário | Resultado exigido |
|---|---|
| Sem `library.json` | Setup obrigatório com mestre e réplica vazios |
| Configuração válida | Dashboard abre e serviços iniciam |
| Mestre desconectado | Tela de reparo; nenhum job iniciado |
| Catálogo ausente | Tela de reparo com causa explícita |
| Réplica desconectada | Tela de reparo; proteção não é simulada |
| Destinos iguais/aninhados | Configuração recusada |
| Fonte removida | Análise recusada antes de criar job |
| Caminhos corrigidos | ID e histórico da biblioteca preservados |

