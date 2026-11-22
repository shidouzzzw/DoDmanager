// SPDX-License-Identifier: GPL-3.0-or-later
//! Bağımlılıksız sözdizimi renklendirme ve markdown üretimi. Tüm giriş önce
//! HTML kaçışından geçer; dosya içeriği asla ham HTML olarak yorumlanmaz.

export function escapeHtml(input: string): string {
  return input
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

const KEYWORDS =
  /\b(fn|let|mut|const|struct|enum|impl|trait|pub|use|mod|match|if|else|for|while|loop|return|break|continue|async|await|move|where|type|import|from|export|default|class|extends|new|this|super|try|catch|finally|throw|interface|function|var|def|end|do|then|elif|and|or|not|in|is|as|static|void|public|private|protected|package|fun|val|when|object|data|override|open|sealed|lateinit)\b/g;

/** Basit ama güvenli renklendirme: yorum, dize, sayı, anahtar sözcük. */
export function highlightCode(code: string, language = ''): string {
  const escaped = escapeHtml(code);
  const useKeywords = !!language && !['txt', 'text', 'log', 'csv'].includes(language.toLowerCase());
  const lines = escaped.split('\n');
  return lines
    .map((line) => {
      let output = line;
      // Yorumlar (//, #, --) satır sonuna kadar.
      output = output.replace(/(^|[^:])(\/\/[^\n]*)/g, '$1<span class="tok-comment">$2</span>');
      output = output.replace(/(^|\s)(#[^\n]*)/g, '$1<span class="tok-comment">$2</span>');
      // Dizeler.
      output = output.replace(
        /(&quot;[^&]*?&quot;|&#39;[^&]*?&#39;)/g,
        '<span class="tok-string">$1</span>',
      );
      // Sayılar.
      output = output.replace(/\b(\d+(\.\d+)?)\b/g, '<span class="tok-number">$1</span>');
      // Anahtar sözcükler (span etiketlerinin içine girmemek için son adım).
      if (useKeywords) {
        output = output.replace(KEYWORDS, '<span class="tok-keyword">$&</span>');
      }
      return output;
    })
    .join('\n')
    .replace(/(<span class="tok-keyword">)(.*?)(<\/span>)/g, '$1$2$3');
}

/** Küçük markdown alt kümesi: başlık, liste, kod, vurgu, bağlantı. */
export function renderMarkdown(markdown: string): string {
  const escaped = escapeHtml(markdown).replace(/\r\n/g, '\n');
  const lines = escaped.split('\n');
  const html: string[] = [];
  let inCode = false;
  let inList = false;
  let paragraph: string[] = [];

  const flushParagraph = () => {
    if (!paragraph.length) return;
    html.push(`<p>${inline(paragraph.join(' '))}</p>`);
    paragraph = [];
  };
  const closeList = () => {
    if (inList) {
      html.push('</ul>');
      inList = false;
    }
  };

  for (const rawLine of lines) {
    const line = rawLine.trimEnd();
    if (line.startsWith('```')) {
      if (inCode) {
        html.push('</code></pre>');
        inCode = false;
      } else {
        flushParagraph();
        closeList();
        html.push('<pre><code>');
        inCode = true;
      }
      continue;
    }
    if (inCode) {
      html.push(`${line}\n`);
      continue;
    }
    if (!line.trim()) {
      flushParagraph();
      closeList();
      continue;
    }
    const heading = /^(#{1,6})\s+(.*)$/.exec(line);
    if (heading) {
      flushParagraph();
      closeList();
      const level = heading[1].length;
      html.push(`<h${level}>${inline(heading[2])}</h${level}>`);
      continue;
    }
    if (/^>\s?/.test(line)) {
      flushParagraph();
      closeList();
      html.push(`<blockquote>${inline(line.replace(/^>\s?/, ''))}</blockquote>`);
      continue;
    }
    if (/^(-{3,}|\*{3,})$/.test(line.trim())) {
      flushParagraph();
      closeList();
      html.push('<hr />');
      continue;
    }
    const listItem = /^\s*[-*+]\s+(.*)$/.exec(line);
    if (listItem) {
      flushParagraph();
      if (!inList) {
        html.push('<ul>');
        inList = true;
      }
      html.push(`<li>${inline(listItem[1])}</li>`);
      continue;
    }
    paragraph.push(line.trim());
  }
  if (inCode) html.push('</code></pre>');
  flushParagraph();
  closeList();
  return html.join('\n');
}

function inline(text: string): string {
  return text
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    .replace(
      /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g,
      '<a href="$2" rel="noopener noreferrer" target="_blank">$1</a>',
    );
}
