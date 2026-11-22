import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(process.cwd(), '..', 'ui');
const apiDir = path.resolve(process.cwd(), 'src', 'api');
const MAX_DEFAULT = 44;

export interface Member {
	name: string;
	type: string;
	default: string;
	access: 'in' | 'in-out' | 'out';
}

export interface Callback {
	name: string;
	arguments: string;
	returns: string;
}

export interface Fn {
	name: string;
	arguments: string;
	returns: string;
}

export interface Component {
	name: string;
	inherits: string;
	properties: Member[];
	callbacks: Callback[];
	functions: Fn[];
}

export interface EnumType {
	kind: 'enum';
	name: string;
	values: string[];
	file: string;
}

export interface StructType {
	kind: 'struct';
	name: string;
	fields: { name: string; type: string }[];
	file: string;
}

const strip = (source: string) => source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');

const matching = (text: string, open: number) => {
	let depth = 0;
	for (let index = open; index < text.length; index++) {
		if (text[index] === '{') depth++;
		if (text[index] === '}' && --depth === 0) return index;
	}
	return text.length;
};

const statements = (body: string) => {
	const found: string[] = [];
	let depth = 0;
	let start = 0;
	let declaration: boolean | undefined;
	for (let index = 0; index < body.length; index++) {
		const char = body[index];
		if (declaration === undefined && /\S/.test(char)) declaration = /^(in-out|in|out|property|private|callback|pure|public)\b/.test(body.slice(index));
		if ('{[('.includes(char)) depth++;
		if ('}])'.includes(char)) depth--;
		const ends = declaration ? depth === 0 && char === ';' : depth === 0 && char === '}';
		if (ends) {
			found.push(body.slice(start, index + 1).trim());
			start = index + 1;
			declaration = undefined;
		}
	}
	return found;
};

const tidy = (text: string) => text.replace(/\s+/g, ' ').trim();

const shorten = (value: string) => {
	const text = tidy(value).replace(/\broot\./g, '');
	return text.length > MAX_DEFAULT ? `${text.slice(0, MAX_DEFAULT - 1)}…` : text;
};

export function parseComponents(source: string, keyword = 'component'): Component[] {
	const text = strip(source);
	const found: Component[] = [];
	for (const match of text.matchAll(new RegExp(`export\\s+${keyword}\\s+([\\w-]+)(?:\\s+inherits\\s+([\\w-]+))?\\s*\\{`, 'g'))) {
		const open = match.index! + match[0].length - 1;
		const body = text.slice(open + 1, matching(text, open));
		const component: Component = { name: match[1], inherits: match[2] ?? '', properties: [], callbacks: [], functions: [] };
		for (const statement of statements(body)) {
			const property = statement.match(/^(in-out|in|out)\s+property\s*<(.+?)>\s*([\w-]+)\s*(?::\s*([\s\S]*?))?;$/);
			if (property) {
				component.properties.push({ access: property[1] as Member['access'], type: tidy(property[2]), name: property[3], default: shorten(property[4] ?? '') });
				continue;
			}
			const callback = statement.match(/^callback\s+([\w-]+)\s*(?:\(([\s\S]*?)\))?\s*(?:->\s*([\w\[\]-]+))?\s*;$/);
			if (callback) {
				component.callbacks.push({ name: callback[1], arguments: tidy(callback[2] ?? ''), returns: callback[3] ?? '' });
				continue;
			}
			const method = statement.match(/^public\s+(?:pure\s+)?function\s+([\w-]+)\s*\(([\s\S]*?)\)\s*(?:->\s*([\w\[\]-]+))?\s*\{/);
			if (method) component.functions.push({ name: method[1], arguments: tidy(method[2]), returns: method[3] ?? '' });
		}
		found.push(component);
	}
	return found;
}

const walk = (directory: string): string[] =>
	fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => (entry.isDirectory() ? walk(path.join(directory, entry.name)) : entry.name.endsWith('.slint') ? [path.join(directory, entry.name)] : []));

let types: Map<string, EnumType | StructType> | undefined;

export function allTypes() {
	if (types) return types;
	types = new Map();
	for (const file of walk(root)) {
		const text = strip(fs.readFileSync(file, 'utf8'));
		const where = path.relative(root, file);
		for (const match of text.matchAll(/export\s+enum\s+([\w-]+)\s*\{([^}]*)\}/g)) {
			types.set(match[1], { kind: 'enum', name: match[1], values: match[2].split(',').map((value) => value.trim()).filter(Boolean), file: where });
		}
		for (const match of text.matchAll(/export\s+struct\s+([\w-]+)\s*\{([^}]*)\}/g)) {
			const fields = match[2].split(',').map((field) => field.trim()).filter(Boolean).map((field) => {
				const [name, ...rest] = field.split(':');
				return { name: name.trim(), type: tidy(rest.join(':')) };
			});
			types.set(match[1], { kind: 'struct', name: match[1], fields, file: where });
		}
	}
	return types;
}

export function typeNames(type: string) {
	return [...type.matchAll(/[A-Z][\w]*/g)].map((match) => match[0]).filter((name) => allTypes().has(name));
}

export function componentsOf(file: string): Component[] {
	const source = fs.readFileSync(path.join(root, 'components', `${file}.slint`), 'utf8');
	return parseComponents(source);
}

export function globalsOf(): Component[] {
	return parseComponents(fs.readFileSync(path.join(root, 'theme.slint'), 'utf8'), 'global');
}

export function referencedTypes(): Set<string> {
	const used = new Set<string>();
	const visit = (type: string) => {
		for (const name of typeNames(type)) {
			if (used.has(name)) continue;
			used.add(name);
			const found = allTypes().get(name);
			if (found?.kind === 'struct') for (const field of found.fields) visit(field.type);
		}
	};
	const files = fs.readdirSync(path.join(root, 'components')).filter((name) => name.endsWith('.slint'));
	for (const file of files) {
		for (const component of componentsOf(file.replace('.slint', ''))) {
			for (const member of component.properties) visit(member.type);
		}
	}
	for (const component of globalsOf()) for (const member of component.properties) visit(member.type);
	return used;
}

export function descriptions(file: string): Record<string, Record<string, string>> {
	const target = path.join(apiDir, `${file}.json`);
	return fs.existsSync(target) ? JSON.parse(fs.readFileSync(target, 'utf8')) : {};
}

export function shared(): Record<string, string> {
	return JSON.parse(fs.readFileSync(path.join(apiDir, '_shared.json'), 'utf8'));
}

export function withInherited(file: string, component: Component, bases: string[]): Component {
	const merged: Component = { ...component, properties: [...component.properties], callbacks: [...component.callbacks], functions: [...component.functions] };
	for (const base of bases) {
		const parent = componentsOf(base).find((candidate) => candidate.name === component.inherits);
		if (!parent) continue;
		const known = new Set(merged.properties.map((member) => member.name));
		merged.properties.push(...parent.properties.filter((member) => !known.has(member.name)));
		merged.callbacks.push(...parent.callbacks);
		merged.functions.push(...parent.functions);
	}
	void file;
	return merged;
}
