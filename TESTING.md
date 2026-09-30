# Estratégia de testes do Lumina

Esta é a fonte vigente de qualidade. Scripts com número de versões antigas permanecem como histórico e empacotamento reproduzível, mas não definem sozinhos o gate atual.

## Validação rápida de desenvolvimento

```powershell
npm.cmd ci
npm.cmd test
npm.cmd run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1
```

Esses comandos detectam regressões comuns. Eles não aprovam budgets de memória, instalador ou endurance.

## Pirâmide obrigatória da 0.22.2

| Camada | Cobertura mínima |
|---|---|
| Unidade | parser de range, limites, estados/transições, retry, sanitização, cache e cálculo de pressão |
| Propriedade/fuzz | ranges, URIs, metadados, nomes, JSON de ferramentas e overflow de tamanhos |
| Integração | SQLite sob concorrência, transações, leases, backup API, processos com saída limitada e filesystem com falhas |
| Componente | lifecycle de foto/vídeo, LRU, cancelamento, seleção e estados de jobs |
| E2E desktop | executável real, WebView2, protocolos, árvore de processos e interação sob carga |
| Fault injection | falta de espaço, fonte/backup offline, kill, timeout, saída excessiva e falha antes/depois de commits |
| Escala/endurance | 100 mil registros, arquivo esparso de 64 GiB, 500/5.000 navegações e soak de 2 h |

## Gates de distribuição

1. Todos os comandos rápidos aprovados.
2. Auditoria de dependências e SBOM sem vulnerabilidade não aceita.
3. MSI e NSIS construídos a partir do commit candidato.
4. Smoke funcional do pacote instalado, não apenas abertura da janela.
5. Upgrade da 0.22.1 e instalação limpa preservam/constroem o catálogo corretamente.
6. Matriz `TRACEABILITY-0.22.2.md` preenchida com artefatos e métricas.
7. Nenhum P0/P1 aberto e todos os requisitos de integridade, privacidade e memória aprovados.
8. O smoke instalado deve concluir setup, validar mestre/réplica e importar um fixture; manter a janela aberta não constitui aprovação.
9. Executar a matriz de estado persistido de `REMEDIATION-0.25.1.md`, incluindo reinstalação, volumes ausentes e garantia de zero jobs órfãos.

## Ambiente mínimo de desempenho

- Windows 10 22H2 ou Windows 11 x64;
- 8 GiB de RAM e 4 processadores lógicos;
- catálogo de 100 mil mídias;
- disco lento e disco rápido registrados separadamente;
- árvore inteira de processos medida pela mesma ferramenta.

Cada relatório registra commit, versões do SO/WebView2/ferramentas, hardware, dataset, duração, p50/p95/p99, pico, residual e hashes dos artefatos. Resultado sem contexto não é comparável.

## Aceite manual seguro

Use a galeria de homologação ou cópias controladas. Preserve originais independentes.

1. Instale limpo e configure acervo/réplica em volumes distintos.
2. Importe fotos, RAW e vídeos; navegue enquanto metadados executam.
3. Pause, cancele, feche à força, reabra e retome cada estágio.
4. Navegue 500 itens, use zoom/comparação, alterne vídeos e faça scroll longo.
5. Teste proteção, fonte/backup offline e falta de espaço.
6. Confirme favoritos, tags, datas, lugares, duplicatas e estados de jobs após reinício.
7. Exporte diagnóstico e verifique que não contém dados pessoais.

Falha de aceite inclui: encerramento inesperado, orçamento excedido, heartbeat acima de 5 s, job sem lease válido, original alterado, proteção falsa, configuração híbrida, cache sem estabilização ou diagnóstico com PII.
