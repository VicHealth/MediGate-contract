#!/usr/bin/env node
/**
 * MediGate - Soroban Contract Deploy Script (Node.js)
 *
 * Deploys the AccessLog contract to Stellar testnet using @stellar/stellar-sdk.
 * This script provides a programmatic alternative to the shell deploy script.
 *
 * Usage:
 *   node deploy.mjs                              # Default testnet deploy
 *   node deploy.mjs --network testnet             # Deploy to testnet
 *   node deploy.mjs --secret-key <key>            # Use specific secret key
 *
 * Environment Variables:
 *   STELLAR_NETWORK       - Network to deploy to (default: testnet)
 *   STELLAR_SECRET_KEY    - Secret key for the deployer account
 *   SOROBAN_RPC_URL       - Soroban RPC endpoint
 */

import { execSync } from 'child_process';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const CONTRACTS_DIR = resolve(__dirname, '..');
const OUTPUT_FILE = resolve(CONTRACTS_DIR, '.contract-ids');

const NETWORK = process.env.STELLAR_NETWORK || 'testnet';
const RPC_URL = process.env.SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org';

// ANSI colors
const colors = {
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  red: '\x1b[31m',
  reset: '\x1b[0m',
};

function log(level, msg) {
  const prefix = {
    info: `${colors.blue}[INFO]${colors.reset}`,
    ok: `${colors.green}[OK]${colors.reset}`,
    warn: `${colors.yellow}[WARN]${colors.reset}`,
    error: `${colors.red}[ERROR]${colors.reset}`,
  };
  console.log(`${prefix[level] || ''} ${msg}`);
}

function run(cmd, opts = {}) {
  return execSync(cmd, {
    cwd: CONTRACTS_DIR,
    encoding: 'utf-8',
    stdio: opts.silent ? 'pipe' : 'inherit',
    ...opts,
  });
}

async function deploy() {
  console.log(`${colors.blue}============================================${colors.reset}`);
  console.log(`${colors.blue}  MediGate Soroban Contract Deployer (Node)${colors.reset}`);
  console.log(`${colors.blue}============================================${colors.reset}`);
  console.log('');
  console.log(`  Network:   ${colors.yellow}${NETWORK}${colors.reset}`);
  console.log(`  RPC URL:   ${colors.yellow}${RPC_URL}${colors.reset}`);
  console.log('');

  // Step 1: Check for Stellar CLI
  try {
    execSync('which stellar', { stdio: 'pipe' });
  } catch {
    log('error', 'Stellar CLI is not installed.');
    log('error', 'Install it from: https://soroban.stellar.org/docs/getting-started/setup');
    process.exit(1);
  }
  log('ok', 'Stellar CLI found');

  // Step 2: Build contracts
  log('info', 'Building contracts...');
  try {
    run('cargo build --target wasm32-unknown-unknown --release', { silent: true });
    log('ok', 'Contracts built successfully');
  } catch (err) {
    log('error', `Build failed: ${err.message}`);
    process.exit(1);
  }

  // Step 3: Deploy AccessLog contract
  log('info', 'Deploying AccessLog contract...');
  const wasmPath = resolve(
    CONTRACTS_DIR,
    'target/wasm32-unknown-unknown/release/access_log.wasm'
  );

  if (!existsSync(wasmPath)) {
    log('error', `WASM file not found at ${wasmPath}`);
    process.exit(1);
  }

  try {
    const deployOutput = run(
      `stellar contract deploy --wasm "${wasmPath}" --network ${NETWORK}`,
      { silent: true }
    );
    const contractId = deployOutput.trim().split('\n').pop();

    log('ok', `AccessLog deployed at: ${contractId}`);

    // Save contract IDs
    const content = [
      `# MediGate Contract IDs (${NETWORK})`,
      `# Deployed at: ${new Date().toISOString()}`,
      `ACCESS_LOG_CONTRACT_ID=${contractId}`,
      '',
    ].join('\n');
    writeFileSync(OUTPUT_FILE, content);
    log('ok', `Contract IDs saved to ${OUTPUT_FILE}`);

    console.log('');
    console.log(`${colors.green}============================================${colors.reset}`);
    console.log(`${colors.green}  Deployment Complete!${colors.reset}`);
    console.log(`${colors.green}============================================${colors.reset}`);
    console.log('');
    console.log(`  AccessLog Contract ID: ${colors.blue}${contractId}${colors.reset}`);
    console.log('');
    console.log(`  View on Stellar Testnet Explorer:`);
    console.log(`  https://stellar.expert/explorer/testnet/contract/${contractId}`);
    console.log('');

    return contractId;
  } catch (err) {
    log('error', `Deployment failed: ${err.message}`);
    process.exit(1);
  }
}

deploy().catch((err) => {
  log('error', err.message);
  process.exit(1);
});
