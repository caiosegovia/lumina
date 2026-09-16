# Segurança e privacidade

## Relato de vulnerabilidades

Não publique vulnerabilidades exploráveis, catálogos reais ou dados pessoais em uma issue pública. Entre em contato com o mantenedor do repositório pelo perfil do GitHub para combinar um canal privado.

## Dados que não devem ser enviados

- Bancos `catalog.sqlite` e diretórios `.lumina`.
- Fotos, vídeos ou sidecars pessoais.
- Logs e relatórios que contenham caminhos ou nomes reais.
- Coordenadas, metadados EXIF e configurações locais.
- Tokens, chaves e arquivos `.env`.

O `.gitignore` cobre os artefatos conhecidos, mas cada alteração deve ser revisada antes do commit.

## Modelo de ameaça da aplicação local

Arquivos de mídia, metadados, nomes, sidecars, respostas de ferramentas e cabeçalhos dos protocolos locais são entradas não confiáveis, mesmo quando vêm do próprio disco. Eles não podem definir tamanhos de alocação, comandos, caminhos fora das raízes autorizadas ou conteúdo executável no WebView.

Controles obrigatórios da estabilização 0.22.2:

- CSP explícita e capabilities Tauri mínimas;
- canonicalização e autorização de toda rota local;
- limite de 64 GiB por arquivo suportado e 4 MiB por resposta de protocolo;
- stdout/stderr de ferramentas drenados com limite;
- processo e descendentes encerrados em timeout/cancelamento;
- configuração, manifestos e relatórios escritos atomicamente;
- diagnóstico estruturado por allowlist, sem caminhos, nomes, GPS, hashes ou segredos.

Os requisitos completos estão em `NFR-0.22.2-RESILIENCE.md`. Um teste negativo ou fuzz que provoque encerramento, escape de raiz ou vazamento de dados bloqueia a release.
