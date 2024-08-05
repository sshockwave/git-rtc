import { hostname } from 'node:os';
import { cwd } from 'process';
import { access, constants, opendir } from 'node:fs/promises';
import { dirname, join, basename } from 'node:path';
import { GitRtcServerInit } from './node-server';

export async function generate_repo_mapping(scan_dir?: string) {
  scan_dir = scan_dir ?? cwd();
  // check if .git exists
  let root_path = scan_dir;
  async function file_exists(path: string) {
    try {
      await access(path, constants.F_OK);
      return true;
    } catch (err) {
      return false;
    }
  }
  while (true) {
    // Check if the file exists in the current directory.
    if (await file_exists(join(root_path, '.git'))) {
      return new Map([['', root_path]]);
    }
    const new_root_path = dirname(root_path);
    if (new_root_path === root_path) {
      break;
    }
    root_path = new_root_path;
  }
  const map = new Map<string, string>();
  for await (const entry of await opendir(scan_dir, {
    encoding: 'utf8',
  })) {
    if (entry.isDirectory() && await file_exists(join(entry.name, '.git'))) {
      map.set(entry.name, join(scan_dir, entry.name));
    }
  }
  return map;
}

export async function generate_default_server_options(options?: GitRtcServerInit): Promise<GitRtcServerInit> {
  const repo_mapping = options?.repo_mapping ?? await generate_repo_mapping();
  const default_repo = repo_mapping.get('');
  const server_name = options?.server_name ?? `${hostname()}${default_repo !== undefined ? `:${basename(default_repo)}` : ''}`;
  return { repo_mapping, server_name };
}
