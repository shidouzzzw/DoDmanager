// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from 'vitest';
import { escapeHtml, highlightCode, renderMarkdown } from './highlight';

describe('escapeHtml', () => {
  it('tehlikeli karakterleri kaçırır', () => {
    expect(escapeHtml('<script>alert(1)</script>')).toBe('&lt;script&gt;alert(1)&lt;/script&gt;');
    expect(escapeHtml('a & b "c"')).toBe('a &amp; b &quot;c&quot;');
  });
});

describe('renderMarkdown', () => {
  it('başlık, liste ve vurguyu üretir', () => {
    const html = renderMarkdown('# Başlık\n\n- bir\n- **iki**\n');
    expect(html).toContain('<h1>Başlık</h1>');
    expect(html).toContain('<ul>');
    expect(html).toContain('<strong>iki</strong>');
  });

  it('ham HTML enjekte etmez', () => {
    const html = renderMarkdown('<img src=x onerror=alert(1)>');
    expect(html).not.toContain('<img');
    expect(html).toContain('&lt;img');
  });

  it('yalnızca http(s) bağlantılarına izin verir', () => {
    const safe = renderMarkdown('[site](https://ornek.dev)');
    expect(safe).toContain('href="https://ornek.dev"');
    const unsafe = renderMarkdown('[kotu](javascript:alert(1))');
    expect(unsafe).not.toContain('href="javascript');
  });

  it('kod bloklarını korur', () => {
    const html = renderMarkdown('```\nconst a = 1;\n```');
    expect(html).toContain('<pre><code>');
    expect(html).toContain('const a = 1;');
  });
});

describe('highlightCode', () => {
  it('anahtar sözcük ve dizeleri işaretler', () => {
    const html = highlightCode('const name = "merhaba"; // yorum', 'ts');
    expect(html).toContain('tok-keyword');
    expect(html).toContain('tok-string');
    expect(html).toContain('tok-comment');
  });

  it('HTML kaçışını korur', () => {
    const html = highlightCode('let x = "<b>";');
    expect(html).not.toContain('<b>');
    expect(html).toContain('&lt;b&gt;');
  });
});
