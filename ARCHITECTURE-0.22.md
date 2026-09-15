# Arquitetura — Lumina 0.22

## Localizações v3

`assets` mantém coordenadas canônicas. O resolvedor em lote lê metadados ricos diretamente da mídia, prioriza campos nativos, usa `Geolocation.dat` apenas como complemento e persiste a evidência em `asset_location_details`. `location_cells` fornece agrupamento e `location_overrides` permanece como camada humana soberana.

```text
Arquivo → ExifTool em lote → normalização → detalhe por ativo
                              ├─ texto embutido
                              ├─ geocoder mundial offline
                              └─ aproximação honesta
                                     ↓
                         célula v3 + override manual
```

O `algorithm_version` permite recalcular somente resultados automáticos obsoletos. A migração transfere um override associado ao ativo para sua nova célula mais precisa.

## Recursos e curadoria

As preferências ficam no catálogo. O backend valida os valores e aplica 1, 2 ou 4 permissões globais de I/O. Interações continuam prioritárias sobre trabalho de fundo. A regra de burst atua somente sobre estado do catálogo e nunca exclui mídia.

## Integração Windows

O arquivo é validado e canonicalizado antes de o Explorer receber `/select,` e o caminho como argumentos distintos. Falhas retornam pelo comando e tornam-se feedback visível.
