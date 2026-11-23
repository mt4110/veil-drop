import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { constants } from 'node:fs';
import { copyFile, cp, mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const target = 'x86_64-unknown-linux-gnu';
const [binaryArg, outputArg] = process.argv.slice(2);

function output(command, args, cwd = root) {
  return execFileSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }).trim();
}

async function digest(path) {
  return createHash('sha256').update(await readFile(path)).digest('hex');
}

function selectedDependencies(metadata) {
  const nodes = new Map(metadata.resolve.nodes.map(node => [node.id, node]));
  const selected = new Set([metadata.resolve.root]);
  const pending = [metadata.resolve.root];
  while (pending.length) {
    for (const dep of nodes.get(pending.pop()).deps) {
      if (!selected.has(dep.pkg) && dep.dep_kinds.some(({ kind }) => kind === null || kind === 'build')) {
        selected.add(dep.pkg);
        pending.push(dep.pkg);
      }
    }
  }
  selected.delete(metadata.resolve.root);
  return metadata.packages.filter(pkg => selected.has(pkg.id)).sort((a, b) => a.id.localeCompare(b.id, 'en'));
}

async function includeNotices(metadata, bundle) {
  const inventory = [];
  for (const pkg of selectedDependencies(metadata)) {
    const directory = dirname(pkg.manifest_path);
    const entries = await readdir(directory, { withFileTypes: true });
    const files = new Set(entries.filter(entry =>
      entry.isFile() && /^(licen[cs]e|copying|notice)([._-].*)?$/i.test(entry.name)
    ).map(entry => join(directory, entry.name)));
    if (pkg.license_file) files.add(resolve(directory, pkg.license_file));
    if (!pkg.license || !files.size) throw new Error(`Review missing license information: ${pkg.name}`);
    const destination = join(bundle, 'third_party', `${pkg.name}-${pkg.version}`);
    await mkdir(destination, { recursive: true });
    for (const file of files) {
      await copyFile(file, join(destination, basename(file)), constants.COPYFILE_EXCL);
    }
    const { name, version, license, source, checksum = null } = pkg;
    inventory.push({ name, version, license, source, checksum });
  }

  const runtime = join(output('rustc', ['--print', 'sysroot']), 'share/doc/rust');
  const destination = join(bundle, 'third_party', 'rust-runtime');
  await mkdir(destination, { recursive: true });
  for (const name of ['COPYRIGHT-library.html', 'COPYRIGHT.html']) {
    await copyFile(join(runtime, name), join(destination, name));
  }
  const licenses = join(runtime, 'licenses');
  for (const name of ['MIT.txt', 'Apache-2.0.txt', 'LLVM-exception.txt']) {
    await readFile(join(licenses, name));
  }
  await cp(licenses, join(destination, 'licenses'), { recursive: true, dereference: true });

  const notices = [
    '# Third-party notices',
    '',
    'Selected normal/build dependencies for the Linux target.',
    'Original license and notice texts are retained in third_party/.',
    'Rust runtime texts and COPYRIGHT-library.html are included separately.',
    'System shared libraries are not redistributed. See INSTALL.md for runtime scope.',
    '',
    ...inventory.map(({ name, version, license }) => `- ${name} ${version}: ${license}`),
    '',
  ];
  await writeFile(join(bundle, 'THIRD_PARTY_NOTICES.md'), notices.join('\n'));
  return inventory;
}

if (!binaryArg || !outputArg || process.argv.length !== 4) {
  throw new Error('Usage: node scripts/package-linux.mjs <binary> <new-output-directory>');
}

const binary = resolve(binaryArg);
const destination = resolve(outputArg);
const header = (await readFile(binary)).subarray(0, 20);
if (header.length !== 20 || header.subarray(0, 6).toString('hex') !== '7f454c460201' ||
    header.readUInt16LE(18) !== 62) {
  throw new Error('Expected little-endian ELF64 x86_64');
}
const metadata = JSON.parse(output('cargo', ['metadata', '--locked', '--format-version', '1', '--filter-platform', target]));
const { name, version } = metadata.packages.find(pkg => pkg.id === metadata.resolve.root);
if (name !== 'veil-drop' || output(binary, ['--version']) !== `veil-drop ${version}`) {
  throw new Error('Binary version differs from package version');
}
if (process.env.GITHUB_REF_TYPE === 'tag' && process.env.GITHUB_REF_NAME !== `v${version}`) {
  throw new Error('Git tag differs from package version');
}
if (output('git', ['status', '--porcelain', '--untracked-files=no'])) {
  throw new Error('Tracked source must be clean before packaging');
}
await mkdir(destination);
const bundle = join(destination, 'bundle');
await mkdir(bundle);
await copyFile(binary, join(bundle, 'veil-drop'));
await copyFile(join(root, 'LICENSE'), join(bundle, 'LICENSE'));
await copyFile(join(root, 'SECURITY.md'), join(bundle, 'SECURITY.md'));
await copyFile(join(root, 'docs/install-linux.md'), join(bundle, 'INSTALL.md'));
const dependencies = await includeNotices(metadata, bundle);
const manifest = {
  source_revision: output('git', ['rev-parse', 'HEAD']),
  target,
  version,
  rustc: output('rustc', ['-vV']),
  dependencies,
  cargo_lock_sha256: await digest(join(root, 'Cargo.lock')),
};
await writeFile(join(bundle, 'BUILD.json'), JSON.stringify(manifest, null, 2) + '\n');

const files = (await readdir(bundle, { recursive: true, withFileTypes: true }))
  .filter(entry => entry.isFile())
  .map(entry => join(entry.parentPath, entry.name).slice(bundle.length + 1))
  .sort();
const sums = await Promise.all(files.map(async file => `${await digest(join(bundle, file))}  ${file}`));
await writeFile(join(bundle, 'SHA256SUMS'), sums.join('\n') + '\n');
const archive = `veil-drop-${version}-${target}.zip`;
output('zip', ['-q', '-X', join(destination, archive), ...files, 'SHA256SUMS'], bundle);
await writeFile(join(destination, 'SHA256SUMS'), `${await digest(join(destination, archive))}  ${archive}\n`);
console.log(`Created ${archive}`);
