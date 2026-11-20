---
name: acessibilidade-frontend
description: "Garante conformidade com WCAG 2.1 AA em interfaces. Use quando: criar componentes, revisar telas."
user-invocable: true
---

# Acessibilidade (a11y) para Frontend React / Next.js

## Quando Usar

- Ao criar qualquer componente ou pagina nova
- Quando o projeto precisar atender a legislacao de acessibilidade (Lei Brasileira n 13.146/2017, ADA, EAA)
- Antes de publicar em producao qualquer interface publica
- Ao revisar PRs de frontend
- Quando receber feedback de usuarios com deficiencia visual, motora, auditiva ou cognitiva

## Procedimento

### Passo 1 — Estrutura Semantica HTML

Use elementos HTML para sua finalidade semantica. Nunca use `<div>` ou `<span>` como substitutos de elementos semanticos.

| Elemento | Uso Correto | Evitar |
|---|---|---|
| `<header>` | Cabecalho da pagina ou de secao | `<div class="header">` |
| `<nav>` | Menu de navegacao principal | `<div class="menu">` |
| `<main>` | Conteudo principal unico por pagina | `<div id="main">` |
| `<article>` | Conteudo independente (post, card de produto) | `<div class="card">` |
| `<section>` | Agrupamento tematico com heading | `<div class="section">` |
| `<aside>` | Conteudo tangencial (sidebar, relacionados) | `<div class="sidebar">` |
| `<footer>` | Rodape da pagina ou de secao | `<div class="footer">` |
| `<button>` | Acoes interativas (abrir modal, enviar form) | `<div onClick>` |
| `<a>` | Navegacao para outra pagina ou ancora | `<div onClick={router.push}>` |

**Regra de ouro:** Se voce clicar em um elemento e ele navega, use `<a>`. Se executa uma acao, use `<button>`.

### Passo 2 — Navegacao por Teclado

Toda funcionalidade deve ser acessivel via teclado (Tab, Enter, Space, Escape, Setas).

```typescript
// ❌ Incorreto — div nao focavel
<div onClick={handleClick}>Abrir menu</div>

// ✅ Correto — button nativamente focavel e ativavel
<button onClick={handleClick}>Abrir menu</button>
```

**Gerenciamento de foco:**
```typescript
import { useRef, useEffect } from 'react';

function Modal({ isOpen, onClose }: ModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);
  const firstFocusableRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (isOpen) {
      firstFocusableRef.current?.focus();
      document.body.style.overflow = 'hidden';
    }
    return () => { document.body.style.overflow = ''; };
  }, [isOpen]);

  // Trap focus dentro do modal
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') onClose();
    // Logica de trap focus omitida para brevidade
  };

  if (!isOpen) return null;

  return (
    <div ref={modalRef} role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <h2 id="modal-title">Titulo do Modal</h2>
      <button ref={firstFocusableRef} onClick={onClose}>Fechar</button>
    </div>
  );
}
```

**Ordem de tabulacao logica:**
- A ordem visual deve corresponder a ordem de tabulacao
- Nao use `tabIndex` positivo (ex: `tabIndex={1}`)
- Use `tabIndex={0}` para tornar elementos focaveis
- Use `tabIndex={-1}` para foco programatico sem inclusao na ordem natural

### Passo 3 — ARIA e Roles

Use ARIA apenas quando HTML semantico nao for suficiente.

```typescript
// ✅ Correto — ARIA para complementar semantica
<button
  aria-expanded={isOpen}
  aria-controls="menu-content"
  aria-haspopup="true"
>
  Menu
</button>
<div id="menu-content" hidden={!isOpen}>
  {/* itens do menu */}
</div>

// ✅ Correto — Alerta ao vivo para notificacoes
<div role="status" aria-live="polite" className="sr-only">
  {notificationMessage}
</div>

// ✅ Correto — Breadcrumb com navegacao
<nav aria-label="Breadcrumb">
  <ol>
    <li><a href="/">Home</a></li>
    <li><a href="/categoria">Categoria</a></li>
    <li aria-current="page">Produto Atual</li>
  </ol>
</nav>
```

**Atributos ARIA comuns:**

| Atributo | Uso |
|---|---|
| `aria-label` | Nome acessivel quando texto visivel nao e suficiente |
| `aria-labelledby` | Referencia a outro elemento como nome |
| `aria-describedby` | Fornece descricao adicional (ex: formato de senha) |
| `aria-expanded` | Estado de expansao de dropdown, accordion |
| `aria-hidden="true"` | Oculta elemento de leitores de tela (icones decorativos) |
| `role="alert"` | Mensagens importantes que interrompem o usuario |
| `role="status"` | Atualizacoes que nao exigem interrupcao |
| `aria-live="polite"` | Anuncia mudancas quando o leitor estiver ocioso |
| `aria-live="assertive"` | Anuncia mudancas imediatamente |

### Passo 4 — Formularios Acessiveis

```typescript
// ✅ Correto — label explicitamente associado
<label htmlFor="email">Email</label>
<input
  id="email"
  type="email"
  aria-required="true"
  aria-describedby="email-error"
  aria-invalid={hasError}
/>
{hasError && <span id="email-error" role="alert">Email invalido</span>}

// ✅ Correto — fieldset para grupos
<fieldset>
  <legend>Preferencias de notificacao</legend>
  <label>
    <input type="checkbox" name="email" />
    Email
  </label>
  <label>
    <input type="checkbox" name="sms" />
    SMS
  </label>
</fieldset>
```

**Regras de formularios:**
- Todo input deve ter um `<label>` associado via `htmlFor`
- Mensagens de erro devem ter `role="alert"` ou `aria-live="assertive"`
- Use `aria-invalid` para indicar campos com erro
- Agrupe campos relacionados com `<fieldset>` e `<legend>`
- Forneca instrucoes de formato via `aria-describedby`

### Passo 5 — Imagens e Midia

```typescript
// ✅ Informativa — alt descritivo
<img src="grafico-vendas.png" alt="Grafico de vendas do primeiro trimestre mostrando crescimento de 23%" />

// ✅ Decorativa — alt vazio
<img src="decorative-line.png" alt="" />

// ✅ Complexa — descricao longa
<figure>
  <img src="organograma.png" alt="Organograma da empresa" />
  <figcaption>Organograma completo da estrutura hierarquica da empresa</figcaption>
</figure>

// ✅ SVG acessivel
<svg role="img" aria-label="Icone de carrinho de compras">
  {/* paths */}
</svg>
```

**Regras de midia:**
- Videos devem ter legendas (closed captions)
- Audio deve ter transcricao
- Conteudo automatico (carrossel, slideshow) deve ter controles de pausa
- Nao use autoplay de audio sem controle de volume/pausa

### Passo 6 — Contraste e Cores

| Cenario | Razao minima (WCAG AA) |
|---|---|
| Texto normal (< 18px) | 4.5:1 |
| Texto grande (>= 18px bold ou >= 24px) | 3:1 |
| Componentes de UI (bordas, icones) | 3:1 |

**Ferramentas de verificacao:**
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- DevTools do Chrome (Lighthouse > Accessibility)
- axe DevTools (extensao de browser)

**Regras de cores:**
- Nunca use cor como unico indicador de estado (ex: erro em vermelho sem texto)
- Sempre combine cor com texto, icone ou padrao
- Forneca modos de alto contraste quando possivel

### Passo 7 — Skip Links e Landmarks

```tsx
// app/layout.tsx
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="pt-BR">
      <body>
        <a href="#main-content" className="skip-link">
          Pular para o conteudo principal
        </a>
        <header>{/* navegacao */}</header>
        <main id="main-content" tabIndex={-1}>
          {children}
        </main>
        <footer>{/* rodape */}</footer>
      </body>
    </html>
  );
}
```

```css
/* styles/globals.css */
.skip-link {
  position: absolute;
  top: -40px;
  left: 0;
  background: #000;
  color: #fff;
  padding: 8px 16px;
  z-index: 100;
  transition: top 0.3s;
}

.skip-link:focus {
  top: 0;
}
```

### Passo 8 — Testes Automatizados

Instale e configure:

```bash
npm install -D @axe-core/react jest-axe
```

```typescript
// __tests__/a11y.test.tsx
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { Button } from '@/components/ui/Button';

expect.extend(toHaveNoViolations);

describe('Button', () => {
  it('nao deve ter violacoes de acessibilidade', async () => {
    const { container } = render(<Button>Clique aqui</Button>);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
```

**Testes manuais obrigatorios:**
- Navegue a pagina inteira usando apenas Tab, Enter, Space, Escape
- Teste com leitor de tela (NVDA, VoiceOver, JAWS)
- Verifique zoom ate 200% sem perda de funcionalidade
- Teste em modo de alto contraste do sistema operacional

### Passo 9 — Checklist WCAG 2.1 AA

- [ ] Todo conteudo tem alternativa textual (imagens, icones, graficos)
- [ ] Videos tem legendas e transcricoes
- [ ] Contraste de texto e componentes atende 4.5:1 (3:1 para grandes)
- [ ] Texto pode ser redimensionado ate 200% sem perda
- [ ] Navegacao por teclado funciona em toda a pagina
- [ ] Foco visivel esta sempre presente e evidente
- [ ] Nao ha armadilhas de teclado (keyboard traps)
- [ ] Links e botoes tem nomes acessiveis descritivos
- [ ] Formularios tem labels, instrucoes e mensagens de erro claras
- [ ] Idioma da pagina esta declarado (`<html lang="pt-BR">`)
- [ ] Skip link para conteudo principal esta presente
- [ ] Landmarks semanticos (header, nav, main, footer) estao presentes
- [ ] Modais e dialogs gerenciam foco corretamente
- [ ] Notificacoes e atualizacoes dinamicas usam aria-live
- [ ] Pagina tem titulo descritivo e unico (`<title>`)

## Output Esperado

- Componentes e paginas com HTML semantico correto
- Atributos ARIA aplicados onde necessario
- Testes automatizados de acessibilidade com jest-axe
- Relatorio de auditoria a11y com violacoes corrigidas e pendencias
- Documentacao de padroes de acessibilidade do projeto
