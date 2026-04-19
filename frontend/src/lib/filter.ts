/**
 * Tiny model + RFC 4515 compiler / decompiler for the visual filter
 * builder. Two value shapes round-trip:
 *
 *   group  =  AND / OR / NOT  with children
 *   leaf   =  single comparison on an attribute
 *
 * The compiler produces canonical RFC 4515 strings (with proper
 * escaping); the parser is intentionally tolerant — it accepts any
 * filter we previously produced and most hand-written ones, but it
 * gives up on extensible match (`:dn:caseIgnoreMatch:=`) etc., in
 * which case the filter is kept as a single "raw" leaf so the UI can
 * still show + ship it.
 */

export type Op = 'and' | 'or' | 'not';
export type LeafOp = 'equal' | 'present' | 'substring' | 'ge' | 'le' | 'approx';

export interface Leaf {
  kind: 'leaf';
  attribute: string;
  op: LeafOp;
  value: string;
  /** When the parser cannot understand a node, it is stashed verbatim. */
  raw?: string;
}

export interface Group {
  kind: 'group';
  op: Op;
  children: Node[];
}

export type Node = Leaf | Group;

export const EMPTY_GROUP = (): Group => ({ kind: 'group', op: 'and', children: [] });
export const EMPTY_LEAF = (): Leaf => ({
  kind: 'leaf',
  attribute: 'objectClass',
  op: 'present',
  value: ''
});

// ---------- compile ----------

/** Escape per RFC 4515 §3: `*`, `(`, `)`, `\` and NUL. */
export function escape4515(value: string): string {
  let out = '';
  for (const ch of value) {
    switch (ch) {
      case '\\':
        out += '\\5c';
        break;
      case '*':
        out += '\\2a';
        break;
      case '(':
        out += '\\28';
        break;
      case ')':
        out += '\\29';
        break;
      case '\0':
        out += '\\00';
        break;
      default:
        out += ch;
    }
  }
  return out;
}

export function compile(node: Node): string {
  if (node.kind === 'group') {
    if (node.op === 'not') {
      const child = node.children[0];
      return `(!${child ? compile(child) : '(objectClass=*)'})`;
    }
    if (node.children.length === 0) return '(objectClass=*)';
    if (node.children.length === 1) return compile(node.children[0]);
    const sym = node.op === 'and' ? '&' : '|';
    return `(${sym}${node.children.map(compile).join('')})`;
  }

  if (node.raw) return node.raw;
  const a = node.attribute.trim();
  if (!a) return '(objectClass=*)';
  switch (node.op) {
    case 'present':
      return `(${a}=*)`;
    case 'equal':
      return `(${a}=${escape4515(node.value)})`;
    case 'substring': {
      // value already contains user-typed `*` markers; we escape every
      // other RFC 4515 special character but keep the wildcard.
      const parts = node.value.split('*').map((p) => escape4515(p));
      return `(${a}=${parts.join('*')})`;
    }
    case 'ge':
      return `(${a}>=${escape4515(node.value)})`;
    case 'le':
      return `(${a}<=${escape4515(node.value)})`;
    case 'approx':
      return `(${a}~=${escape4515(node.value)})`;
  }
}

// ---------- parse ----------

class Parser {
  src: string;
  i = 0;
  constructor(s: string) {
    this.src = s.trim();
  }

  parse(): Node {
    this.expect('(');
    const node = this.parseInside();
    this.expect(')');
    if (this.i !== this.src.length) throw new Error('trailing garbage');
    return node;
  }

  parseInside(): Node {
    const ch = this.peek();
    if (ch === '&' || ch === '|') {
      const op: Op = ch === '&' ? 'and' : 'or';
      this.i++;
      const children: Node[] = [];
      while (this.peek() === '(') {
        this.i++;
        children.push(this.parseInside());
        this.expect(')');
      }
      return { kind: 'group', op, children };
    }
    if (ch === '!') {
      this.i++;
      this.expect('(');
      const child = this.parseInside();
      this.expect(')');
      return { kind: 'group', op: 'not', children: [child] };
    }
    return this.parseLeaf();
  }

  parseLeaf(): Leaf {
    // Consume up to `=` (or `>=`, `<=`, `~=`) before the closing `)`.
    let attr = '';
    while (this.i < this.src.length) {
      const c = this.src[this.i];
      if (c === '=' || c === '<' || c === '>' || c === '~' || c === ':') break;
      attr += c;
      this.i++;
    }

    let op: LeafOp = 'equal';
    const c = this.src[this.i];
    if (c === '>') {
      op = 'ge';
      this.i++;
      this.expect('=');
    } else if (c === '<') {
      op = 'le';
      this.i++;
      this.expect('=');
    } else if (c === '~') {
      op = 'approx';
      this.i++;
      this.expect('=');
    } else if (c === ':') {
      // Extensible match — bail out to raw.
      const start = this.findClose() ?? this.src.length;
      const raw = '(' + this.src.slice(this.captureStart, start) + ')';
      this.i = start;
      return {
        kind: 'leaf',
        attribute: attr,
        op: 'equal',
        value: '',
        raw
      };
    } else {
      this.expect('=');
    }

    let value = '';
    while (this.i < this.src.length && this.src[this.i] !== ')') {
      value += this.src[this.i];
      this.i++;
    }

    if (op === 'equal' && value === '*') {
      return { kind: 'leaf', attribute: attr, op: 'present', value: '' };
    }
    if (op === 'equal' && value.includes('*')) {
      return {
        kind: 'leaf',
        attribute: attr,
        op: 'substring',
        value: unescape4515(value)
      };
    }
    return { kind: 'leaf', attribute: attr, op, value: unescape4515(value) };
  }

  captureStart = 0;
  findClose(): number | null {
    for (let j = this.i; j < this.src.length; j++) {
      if (this.src[j] === ')') return j;
    }
    return null;
  }

  peek(): string {
    return this.src[this.i] ?? '';
  }

  expect(ch: string): void {
    if (this.src[this.i] !== ch) {
      throw new Error(`expected '${ch}' at offset ${this.i}, got '${this.src[this.i] ?? 'EOF'}'`);
    }
    this.i++;
  }
}

function unescape4515(value: string): string {
  return value.replace(/\\([0-9a-fA-F]{2})/g, (_, h: string) =>
    String.fromCharCode(parseInt(h, 16))
  );
}

/**
 * Best-effort parser. Returns `null` if the filter does not start with
 * `(` — the UI then falls back to keeping the raw text.
 */
export function tryParse(filter: string): Node | null {
  const trimmed = filter.trim();
  if (!trimmed.startsWith('(')) return null;
  try {
    return new Parser(trimmed).parse();
  } catch {
    return null;
  }
}
