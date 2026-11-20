---
name: seo-frontend
description: "Otimiza aplicacoes React e Next.js para SEO tecnico e. Use quando: criar ou revisar paginas publicas."
user-invocable: true
---

# SEO para Frontend React / Next.js

## Quando Usar

- Ao criar ou revisar qualquer pagina publica acessivel por mecanismos de busca
- Quando o projeto envolve landing pages, blogs, e-commerce, marketplaces ou diretorios
- Antes de publicar em producao qualquer rota publica
- Quando for necessario configurar sitemap, robots.txt ou structured data
- Ao diagnosticar queda de trafego organico ou problemas de indexacao

## Procedimento

### Passo 1 — Auditoria SEO Tecnica

Verifique os seguintes pontos em cada pagina publica:

| Item | Regra | Ferramenta / Codigo |
|---|---|---|
| Title tag | Unico por pagina, 50-60 caracteres, palavra-chave no inicio | `<title>` ou `next/head` |
| Meta description | 150-160 caracteres, CTA incluso | `<meta name="description">` |
| Canonical URL | Sempre presente, sem parametros de rastreamento | `<link rel="canonical">` |
| Open Graph | `og:title`, `og:description`, `og:image`, `og:url` | `<meta property="og:*">` |
| Twitter Cards | `twitter:card`, `twitter:title`, `twitter:image` | `<meta name="twitter:*">` |
| Structured Data | JSON-LD para Article, Product, FAQ, Breadcrumb, Organization | `<script type="application/ld+json">` |
| Robots | `index, follow` por padrao; `noindex` para paginas internas | `<meta name="robots">` |
| Hreflang | Quando houver multi-idioma | `<link rel="alternate" hreflang="x">` |
| Imagens | Alt text descritivo, formato moderno (WebP/AVIF), lazy loading | `<img alt="..." loading="lazy">` |
| Links internos | Anchor text descritivo, sem genericos como "clique aqui" | `<a href="...">` |

### Passo 2 — Configuracao Next.js (App Router)

Use o metadata API nativo do Next.js 13+:

```typescript
// app/layout.tsx
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: {
    template: '%s | Nome da Marca',
    default: 'Nome da Marca — Slogan ou proposta de valor',
  },
  description: 'Descricao padrao do site em ate 160 caracteres.',
  metadataBase: new URL('https://www.exemplo.com'),
  openGraph: {
    type: 'website',
    locale: 'pt_BR',
    siteName: 'Nome da Marca',
    images: ['/og-default.jpg'],
  },
  twitter: {
    card: 'summary_large_image',
    site: '@marca',
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  alternates: {
    canonical: '/',
    languages: {
      'en-US': '/en',
      'pt-BR': '/pt',
    },
  },
};
```

Para paginas dinamicas:

```typescript
// app/blog/[slug]/page.tsx
import type { Metadata } from 'next';

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const post = await fetchPost(params.slug);
  return {
    title: post.title,
    description: post.excerpt,
    openGraph: {
      title: post.title,
      description: post.excerpt,
      images: [post.coverImage],
      type: 'article',
      publishedTime: post.publishedAt,
      authors: [post.author.name],
    },
    alternates: {
      canonical: `/blog/${post.slug}`,
    },
  };
}
```

### Passo 3 — Structured Data (JSON-LD)

Crie um componente reutilizavel:

```typescript
// components/JsonLd.tsx
import Script from 'next/script';

interface JsonLdProps {
  data: Record<string, unknown>;
}

export function JsonLd({ data }: JsonLdProps) {
  return (
    <Script
      id="json-ld"
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(data) }}
    />
  );
}
```

Exemplos de schemas comuns:

**Article (blog/posts):**
```json
{
  "@context": "https://schema.org",
  "@type": "Article",
  "headline": "Titulo do Artigo",
  "description": "Resumo do artigo",
  "image": "https://www.exemplo.com/imagem.jpg",
  "author": {
    "@type": "Person",
    "name": "Nome do Autor"
  },
  "publisher": {
    "@type": "Organization",
    "name": "Nome da Marca",
    "logo": {
      "@type": "ImageObject",
      "url": "https://www.exemplo.com/logo.png"
    }
  },
  "datePublished": "2024-01-15T08:00:00-03:00",
  "dateModified": "2024-01-15T08:00:00-03:00"
}
```

**Product (e-commerce):**
```json
{
  "@context": "https://schema.org",
  "@type": "Product",
  "name": "Nome do Produto",
  "image": ["https://www.exemplo.com/produto-1.jpg"],
  "description": "Descricao do produto",
  "brand": {
    "@type": "Brand",
    "name": "Marca"
  },
  "offers": {
    "@type": "Offer",
    "url": "https://www.exemplo.com/produto",
    "priceCurrency": "BRL",
    "price": "199.90",
    "availability": "https://schema.org/InStock"
  },
  "aggregateRating": {
    "@type": "AggregateRating",
    "ratingValue": "4.5",
    "reviewCount": "128"
  }
}
```

**FAQPage:**
```json
{
  "@context": "https://schema.org",
  "@type": "FAQPage",
  "mainEntity": [
    {
      "@type": "Question",
      "name": "Pergunta 1?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Resposta 1."
      }
    }
  ]
}
```

### Passo 4 — Sitemap e Robots

**Sitemap dinamico (Next.js):**
```typescript
// app/sitemap.ts
import { MetadataRoute } from 'next';

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const posts = await fetchPosts();
  
  const postUrls = posts.map((post) => ({
    url: `https://www.exemplo.com/blog/${post.slug}`,
    lastModified: post.updatedAt,
    changeFrequency: 'weekly' as const,
    priority: 0.8,
  }));

  return [
    { url: 'https://www.exemplo.com', lastModified: new Date(), changeFrequency: 'daily', priority: 1 },
    { url: 'https://www.exemplo.com/blog', lastModified: new Date(), changeFrequency: 'daily', priority: 0.9 },
    ...postUrls,
  ];
}
```

**Robots.txt:**
```
# public/robots.txt
User-agent: *
Allow: /
Disallow: /api/
Disallow: /admin/
Disallow: /checkout/

Sitemap: https://www.exemplo.com/sitemap.xml
```

### Passo 5 — Core Web Vitals e Performance

| Metrica | Objetivo | Acao no React/Next.js |
|---|---|---|
| LCP (Largest Contentful Paint) | < 2.5s | Otimizar imagens com `next/image`, preload de fontes criticas |
| INP (Interaction to Next Paint) | < 200ms | Code splitting, lazy loading de componentes pesados |
| CLS (Cumulative Layout Shift) | < 0.1 | Definir width/height em imagens e iframes, evitar injecao de conteudo acima da dobra |
| TTFB (Time to First Byte) | < 600ms | Edge caching, ISR, SSR otimizado |

**Otimizacoes obrigatorias:**
- Use `next/image` com `priority` para imagens acima da dobra
- Use `next/font` para carregamento otimizado de fontes
- Implemente ISR (`revalidate`) para paginas estaticas que mudam com frequencia
- Prefira Server Components para reduzir JS no cliente
- Use `loading="lazy"` para imagens abaixo da dobra

### Passo 6 — Checklist Pre-Deploy

- [ ] Todas as paginas publicas tem title e description unicos
- [ ] Canonical URL configurado em todas as paginas
- [ ] Open Graph e Twitter Cards presentes
- [ ] Structured Data validado no [Google Rich Results Test](https://search.google.com/test/rich-results)
- [ ] Sitemap gerado e acessivel em `/sitemap.xml`
- [ ] Robots.txt configurado e apontando para o sitemap
- [ ] Imagens com alt text descritivo
- [ ] URLs amigaveis (sem IDs numericos expostos, sem query strings desnecessarias)
- [ ] Pagina 404 personalizada com status HTTP 404
- [ ] Redirecionamentos 301 para URLs antigas ou movidas
- [ ] Core Web Vitals dentro dos limites no PageSpeed Insights

## Output Esperado

- Paginas otimizadas com metadata completo e unico
- Componente `JsonLd` reutilizavel para structured data
- Arquivos `sitemap.ts` e `robots.txt` configurados
- Relatorio de auditoria SEO com acoes realizadas e pendencias
- Recomendacoes de palavras-chave e estrutura de URLs
