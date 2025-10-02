import { startAnchor } from "solana-bankrun";

export const setupBankrun = async () => {
  const context = await startAnchor("./", [], []);
  return context;
};

export const teardownBankrun = async (context: any) => {
  // Bankrun cleanup is handled automatically
  // No explicit teardown needed
};
