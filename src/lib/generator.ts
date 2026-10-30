/**
 * Cryptographically secure, customizable password generator.
 */

export interface PasswordGeneratorOptions {
  /** Total length of the generated password. Default: 20 */
  length?: number;
  /** Include lowercase alphabetic characters (a-z). Default: true */
  lowercase?: boolean;
  /** Include uppercase alphabetic characters (A-Z). Default: true */
  uppercase?: boolean;
  /** Include numerical digits (0-9). Default: true */
  numbers?: boolean;
  /** Include special symbols (!@#$%^&*...). Default: true */
  symbols?: boolean;
  /** Optional custom characters to include in the character pool. */
  customCharset?: string;
  /** Exclude visually similar characters (e.g. i, l, 1, I, o, 0, O). Default: false */
  excludeSimilar?: boolean;
  /** Exclude ambiguous or hard-to-type punctuation (e.g. {}[]()/\'"`~,;:.<>). Default: false */
  excludeAmbiguous?: boolean;
  /** Minimum number of lowercase characters to include. Default: 1 */
  minLowercase?: number;
  /** Minimum number of uppercase characters to include. Default: 1 */
  minUppercase?: number;
  /** Minimum number of numeric digits to include. Default: 1 */
  minNumbers?: number;
  /** Minimum number of symbols to include. Default: 1 */
  minSymbols?: number;
}

const LOWERCASE_CHARS = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMBER_CHARS = "0123456789";
const SYMBOL_CHARS = "!@#$%^&*()_+-=[]{}|;:,.<>?";

const SIMILAR_CHARS = new Set(["i", "l", "1", "I", "o", "0", "O"]);
const AMBIGUOUS_CHARS = new Set([
  "{",
  "}",
  "[",
  "]",
  "(",
  ")",
  "/",
  "\\",
  "'",
  '"',
  "`",
  "~",
  ",",
  ";",
  ":",
  ".",
  "<",
  ">",
]);

export const DEFAULT_GENERATOR_OPTIONS: Required<
  Omit<PasswordGeneratorOptions, "customCharset">
> & { customCharset?: string } = {
  length: 20,
  lowercase: true,
  uppercase: true,
  numbers: true,
  symbols: true,
  customCharset: "",
  excludeSimilar: false,
  excludeAmbiguous: false,
  minLowercase: 1,
  minUppercase: 1,
  minNumbers: 1,
  minSymbols: 1,
};

/**
 * Generate a random integer in [0, maxExclusive) using rejection sampling to eliminate modulo bias.
 */
function getRandomInt(maxExclusive: number): number {
  if (maxExclusive <= 0) return 0;
  const threshold = Math.floor(0x100000000 / maxExclusive) * maxExclusive;
  const buffer = new Uint32Array(1);
  while (true) {
    crypto.getRandomValues(buffer);
    if (buffer[0] < threshold) {
      return buffer[0] % maxExclusive;
    }
  }
}

function filterCharset(
  chars: string,
  excludeSimilar: boolean,
  excludeAmbiguous: boolean,
): string {
  let result = "";
  for (const c of chars) {
    if (excludeSimilar && SIMILAR_CHARS.has(c)) continue;
    if (excludeAmbiguous && AMBIGUOUS_CHARS.has(c)) continue;
    result += c;
  }
  return result;
}

/**
 * Generate a cryptographically secure random password based on the given configuration.
 */
export function generatePassword(
  options: PasswordGeneratorOptions = {},
): string {
  const opts = { ...DEFAULT_GENERATOR_OPTIONS, ...options };
  const targetLength = Math.max(1, opts.length);

  // Filter individual charsets according to exclusions
  const lower = opts.lowercase
    ? filterCharset(LOWERCASE_CHARS, opts.excludeSimilar, opts.excludeAmbiguous)
    : "";
  const upper = opts.uppercase
    ? filterCharset(UPPERCASE_CHARS, opts.excludeSimilar, opts.excludeAmbiguous)
    : "";
  const nums = opts.numbers
    ? filterCharset(NUMBER_CHARS, opts.excludeSimilar, opts.excludeAmbiguous)
    : "";
  const syms = opts.symbols
    ? filterCharset(SYMBOL_CHARS, opts.excludeSimilar, opts.excludeAmbiguous)
    : "";
  const custom = opts.customCharset
    ? filterCharset(
        opts.customCharset,
        opts.excludeSimilar,
        opts.excludeAmbiguous,
      )
    : "";

  // Combine available characters
  const fullPool = lower + upper + nums + syms + custom;
  if (!fullPool) {
    throw new Error(
      "Cannot generate password: character pool is empty with the current options.",
    );
  }

  const resultChars: string[] = [];

  // Helper to append minimum required characters from a specific pool
  const addRequired = (pool: string, minCount: number) => {
    if (!pool || minCount <= 0) return;
    for (let i = 0; i < minCount && resultChars.length < targetLength; i++) {
      const idx = getRandomInt(pool.length);
      resultChars.push(pool[idx]);
    }
  };

  if (opts.lowercase) addRequired(lower, opts.minLowercase);
  if (opts.uppercase) addRequired(upper, opts.minUppercase);
  if (opts.numbers) addRequired(nums, opts.minNumbers);
  if (opts.symbols) addRequired(syms, opts.minSymbols);

  // Fill remaining characters uniformly from full pool
  while (resultChars.length < targetLength) {
    const idx = getRandomInt(fullPool.length);
    resultChars.push(fullPool[idx]);
  }

  // Fisher-Yates shuffle with cryptographic randomness
  for (let i = resultChars.length - 1; i > 0; i--) {
    const j = getRandomInt(i + 1);
    const temp = resultChars[i];
    resultChars[i] = resultChars[j];
    resultChars[j] = temp;
  }

  return resultChars.join("");
}
