---
name: scaffold-compose-saas
description: "Cria a estrutura de microsserviço isolado para novas. Use quando: gerar novas aplicações SaaS."
user-invocable: true
---

# Skill Scaffold Compose SaaS — Microsserviços Isolados com Docker Compose

Esta skill define o padrão arquitetural obrigatório para toda nova aplicação, plataforma ou SaaS gerado pelo J.A.R.V.I.S. ou agentes do ecossistema PE-AGENTS-IBM. Cada aplicação nasce como um **microsserviço independente**, contendo seu próprio `Dockerfile` e `docker-compose.yml`, eliminando qualquer acoplamento com o servidor central do HUD (porta 8085).

---

## Estrutura Canônica de Diretórios (`platforms/<slug>/`)

```
platforms/<slug>/
├── Dockerfile              # Imagem Alpine Nginx para frontends ou multi-stage para backends
├── docker-compose.yml      # Declaração do serviço isolado e mapeamento de portas
├── index.html              # Frontend da aplicação (React/Tailwind/Carbon)
├── assets/                 # Imagens, ícones e recursos estáticos
├── .gitignore              # Bloqueio de .env, target/, node_modules/ e secrets
├── .gitlab-ci.yml          # Esteira CI/CD obrigatória (GitLab CE 198.51.100.10)
└── nginx.conf              # Configuração opcional de proxy reverso e SPA fallback
```

---

## Padrão Canônico do Dockerfile (Alpine Nginx)

```dockerfile
FROM nginx:alpine
WORKDIR /usr/share/nginx/html
RUN rm -rf ./*
COPY index.html ./index.html
COPY . .
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

---

## Padrão Canônico do docker-compose.yml

```yaml
version: '3.8'

services:
  web:
    build: .
    container_name: saas_<slug>
    ports:
      - "${HOST_PORT:-8200}:80"
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "wget -q --spider http://127.0.0.1/ || exit 1"]
      interval: 10s
      timeout: 3s
      retries: 3
```

---

## Protocolo de Alocação de Portas

1. **Faixa Dedicada**: Portas no intervalo `8200..=8900`.
2. **Verificação de Ocupação**: O motor Rust testa dinamicamente se a porta está livre antes de subir o compose.
3. **Desacoplamento do HUD**: O HUD Python (porta 8085) não serve arquivos estáticos de `platforms/`. Apenas descobre e encaminha a URL `http://localhost:<porta>` ou `http://<ip-lan>:<porta>` para renderização.

---

## Premissa Obrigatória: Repositório GitLab + Esteira CI/CD + Pergunta de Backup no GitHub

Todo projeto ou plataforma gerado (seja diretamente via `SaasFactory`/`PlatformBuilder` ou por agentes) DEVE nascer com:
1. **Repositório no GitLab Local (`198.51.100.10`)**: `git init -b main` e remote `origin` apontando para `git@198.51.100.10:your-org/<slug>.git`.
2. **Esteira `.gitlab-ci.yml` + `.gitignore` na raiz**: pipeline CI/CD configurada desde o nascimento (`validate`, `build`, `security`, `deploy`).
3. **Pergunta Obrigatória de Backup no GitHub**: Sempre pergunte ao usuário: *"Deseja criar também um repositório de backup/espelho no GitHub para este projeto?"*. Se confirmado, crie o repositório no GitHub (`gh repo create your-org/<slug> --private`) e configure o remote secundário `github` (ou push duplo no `origin`).
