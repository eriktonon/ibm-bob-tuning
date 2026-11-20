---
name: dockerfile-review
description: "Revisa Dockerfiles existentes em busca de problemas de. Use quando: receber um Dockerfile para avaliar."
user-invocable: true
---

# Revisão de Dockerfile

## Quando Usar

- Antes de publicar qualquer imagem em produção
- Ao receber um Dockerfile de terceiros ou de repositório legado
- Após alterações significativas na aplicação que impactem o build
- Como parte do processo de onboarding de novos serviços

## Procedimento

### Passo 1 — Ler o Dockerfile

Leia o arquivo com atenção antes de iniciar qualquer avaliação. Identifique: runtime, número de stages, imagem base, comandos de build e instrução final de execução.

### Passo 2 — Executar checklist por categoria

---

## Categoria 1 — Imagem Base

- [ ] A imagem base tem versão exata pinada? (`node:20.18-alpine3.20`, não `node:latest` ou `node:20`)
- [ ] A imagem base é mínima? (`alpine`, `slim`, `distroless` são preferíveis a imagens completas)
- [ ] A imagem de runtime é diferente da imagem de build? (multi-stage obrigatório para apps compiladas)
- [ ] A imagem base tem vulnerabilidades conhecidas? (verificar via `docker scout` ou `trivy image`)

**Problemas frequentes:**
```dockerfile
# ❌ Tag mutável — pode mudar silenciosamente
FROM node:latest

# ❌ Imagem de build usada como runtime (inclui compilador, sdks, etc.)
FROM node:20 AS builder
# ... sem segundo stage

# ✅ Imagem mínima com versão exata + multi-stage
FROM node:20.18-alpine3.20 AS builder
# ...
FROM node:20.18-alpine3.20 AS runtime
```

---

## Categoria 2 — Segurança

- [ ] O processo final roda como usuário não-root?
- [ ] `USER` foi definido antes do `CMD`/`ENTRYPOINT`?
- [ ] Há secrets, senhas ou tokens em variáveis `ENV` ou `ARG`?
- [ ] O `.dockerignore` existe e exclui `.env`, `.git`, `node_modules`, `target/`, `__pycache__`?
- [ ] `COPY . .` copia arquivos desnecessários (source de CI, .DS_Store, etc.)?
- [ ] `--no-cache` é usado em `apt-get`/`apk` para não armazenar índices?

**Problemas frequentes:**
```dockerfile
# ❌ Rodando como root (default)
COPY app .
CMD ["node", "server.js"]

# ❌ Secret exposto em variável
ENV DATABASE_PASSWORD=mysecret123

# ✅ Usuário não-root criado e usado
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser
CMD ["node", "server.js"]

# ✅ apt-get sem cache
RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
```

---

## Categoria 3 — Otimização de Cache e Tamanho

- [ ] As dependências (`package.json`, `requirements.txt`, `pom.xml`) são copiadas ANTES do código-fonte?
- [ ] O build é feito em stage separado do runtime?
- [ ] Arquivos desnecessários de build não são copiados para o stage final?
- [ ] Múltiplos `RUN` consecutivos de `apt-get`/`apk` são combinados em um único `RUN`?
- [ ] `npm install` usa `--ci` (reproduzível) em vez de `npm install` puro?

**Problemas frequentes:**
```dockerfile
# ❌ Código copiado antes das dependências — invalida cache a cada mudança de código
COPY . .
RUN npm install

# ✅ Dependências primeiro — cache reutilizado se package.json não mudar
COPY package*.json ./
RUN npm ci --only=production
COPY src/ ./src/

# ❌ Dois RUN separados criam duas camadas
RUN apt-get update
RUN apt-get install -y curl

# ✅ Um RUN = uma camada
RUN apt-get update && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
```

---

## Categoria 4 — Observabilidade e Operação

- [ ] `HEALTHCHECK` está definido com `--interval`, `--timeout`, `--start-period` e `--retries`?
- [ ] O health check aponta para um endpoint real e implementado na aplicação (`/health`)?
- [ ] `EXPOSE` declara a porta correta?
- [ ] Há `LABEL` com metadados mínimos (`version`, `maintainer`, `description`)?
- [ ] A aplicação captura `SIGTERM` e faz graceful shutdown?

**Template de HEALTHCHECK:**
```dockerfile
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD wget -qO- http://localhost:3000/health || exit 1
```

---

## Categoria 5 — Determinismo de Build

- [ ] O build é reproduzível? (mesmas entradas → mesma imagem)
- [ ] Versões de dependências estão travadas (`package-lock.json`, `poetry.lock`, `Gemfile.lock`)?
- [ ] `npm install` / `pip install` sem versão pinada está ausente do Dockerfile?
- [ ] Downloads de scripts externos via `curl | bash` estão ausentes?

**Problemas frequentes:**
```dockerfile
# ❌ Download de script externo sem verificação de hash
RUN curl -fsSL https://get.example.com | bash

# ❌ Versão de dependência não travada
RUN pip install flask   # qual versão?

# ✅ Lock file copiado e usado
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
```

---

## Passo 3 — Classificar achados

| Severidade | Critério |
|---|---|
| Crítica | Secret exposto, root em produção, imagem base com CVE crítico |
| Alta | Sem multi-stage, sem `.dockerignore`, sem HEALTHCHECK |
| Média | Cache mal organizado, imagem desnecessariamente grande, sem versão pinada |
| Baixa | Falta de LABEL, EXPOSE errado, style inconsistente |

## Passo 4 — Corrigir e reportar

- Corrija todos os achados **Críticos** e **Altos** antes de considerar pronto
- Para achados **Médios** e **Baixos**, documente como pendência
- Apresente a versão corrigida do Dockerfile com comentários explicando cada mudança

## Passo 5 — Estimar impacto no tamanho da imagem

Antes e depois das correções, estime ou meça o tamanho da imagem:
```bash
docker build -t app:before .
docker images app:before --format "{{.Size}}"
```

---

## Passo 6 — Scan de Vulnerabilidades com Trivy

Após corrigir o Dockerfile, execute o scan de segurança na imagem gerada. O `trivy` é um scanner open-source (Apache-2.0) que detecta CVEs, misconfigurations, secrets e gera SBOM.

### Pré-requisitos

```bash
# Verificar se trivy está instalado
trivy --version || echo "Instale: https://trivy.dev/latest/docs/getting-started/installation/"
```

### Scan de imagem (CVEs)

```bash
# Scan completo da imagem — exit code 1 se houver CRITICAL
trivy image --severity HIGH,CRITICAL --exit-code 1 app:latest

# Scan com formato JSON para integração CI
trivy image --format json --output trivy-report.json app:latest

# Ignorar vulnerabilidades não corrigíveis (ex: distroless sem fix)
trivy image --ignore-unfixed app:latest
```

### Scan de misconfigurations no Dockerfile

```bash
# Trivy também audita o Dockerfile como IaC
trivy config Dockerfile --severity HIGH,CRITICAL
```

### Classificação de achados Trivy

| Severidade Trivy | Ação |
|---|---|
| `CRITICAL` | Bloqueia publicação — corrigir ou atualizar imagem base |
| `HIGH` | Corrigir antes de produção; documentar se não houver fix |
| `MEDIUM` | Documentar como pendência |
| `LOW` | Informativo |

### Remediação comum

```dockerfile
# Se trivy reportar CVEs na imagem base, atualizar para versão patched:
# ❌ FROM node:20.18-alpine3.20  (CVE conhecido)
# ✅ FROM node:20.18-alpine3.21  (patched)

# Para distroless, usar a versão não-devel:
# ✅ FROM gcr.io/distroless/nodejs20-debian12:nonroot
```

> **Nota:** Se `trivy` não estiver disponível, use `docker scout` (Docker Desktop integrado) ou documente a ausência de scan como pendência de segurança.

---

## Output Esperado

- Dockerfile corrigido com todos os achados Críticos e Altos resolvidos
- Lista de achados com severidade, linha e descrição
- `.dockerignore` criado ou atualizado
- Estimativa de redução de tamanho da imagem (quando aplicável)
- Relatório de scan do trivy (quando disponível) com CRITICAL/HIGH resolvidos
- Pendências documentadas para achados Médios e Baixos
