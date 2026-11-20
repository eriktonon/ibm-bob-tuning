---
name: scaffold-react-native
description: "Configura projetos React Native com Expo ou React. Use quando: iniciar um app mobile novo."
user-invocable: true
---

# Scaffold React Native

## Quando Usar

- Ao iniciar um projeto mobile do zero
- Ao converter um app React Native ad hoc em uma base padronizada
- Antes de implementar features, navegacao e integracoes principais

## Procedimento

### Passo 1 — Escolher runtime

Use **Expo** como padrao quando o app nao exigir modulos nativos customizados fora do ecossistema Expo.

Use **React Native CLI** quando o projeto exigir controle total de `android/` e `ios/`, SDKs nativos com configuracao manual, ou bibliotecas que nao funcionem bem via config plugins.

### Passo 2 — Criar o projeto base

#### Expo (padrao)
```bash
npx create-expo-app@latest {nome-projeto} --template blank-typescript
cd {nome-projeto}
```

#### React Native CLI
```bash
npx react-native@latest init {nome-projeto} --template react-native-template-typescript
cd {nome-projeto}
```

### Passo 3 — Instalar dependencias base

```bash
npm install @react-navigation/native @react-navigation/native-stack
npm install @tanstack/react-query axios zustand zod react-hook-form @hookform/resolvers
npm install react-native-safe-area-context react-native-screens
```

Persistencia e ambiente:

```bash
npm install @react-native-async-storage/async-storage react-native-mmkv
```

Expo:

```bash
npx expo install expo-secure-store expo-linking expo-constants
```

React Native CLI:

```bash
npm install react-native-keychain react-native-config
```

### Passo 4 — Estrutura sugerida

```text
src/
├── app/                # bootstrap, providers, navegacao raiz
├── features/           # modulos por feature
├── components/         # UI compartilhada
├── hooks/              # hooks reutilizaveis
├── services/           # HTTP, storage, notificacoes, analytics
├── stores/             # Zustand ou stores globais
├── domain/             # entidades, casos de uso, contratos
├── infra/              # adaptadores e implementacoes concretas
├── types/              # tipos compartilhados
└── utils/              # funcoes puras
```

### Passo 5 — Navegacao e providers

- Configure container de navegacao raiz
- Isole rotas publicas e autenticadas
- Centralize providers de query, tema, sessao e erros no bootstrap do app

### Passo 6 — HTTP e ambiente

- Criar instancia HTTP centralizada
- Definir `.env.example` com URL de API e flags mobile
- Separar configuracao por ambiente sem hardcode de secrets

### Passo 7 — Validacoes iniciais

- Executar typecheck
- Executar lint
- Garantir que o app sobe no simulador/emulador ou Expo Go

## Output Esperado

- Projeto React Native inicializado com stack coerente
- Estrutura de pastas base criada
- Dependencias nucleares instaladas
- Navegacao, HTTP e estado inicial preparados
- `.env.example` criado com placeholders adequados
- Repositório inicializado no GitLab local (`git@198.51.100.10:your-org/<slug>.git`) com `.gitignore` e esteira `.gitlab-ci.yml`
- Pergunta obrigatória realizada ao usuário: *"Deseja criar também um repositório de backup/espelho no GitHub para este projeto?"*
