const puppeteer = require('puppeteer');
const readline = require('readline');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

(async () => {
  const userDataDir = process.argv[2];
  const browser = await puppeteer.launch({
    userDataDir,
    headless: true,
  });
  const page = (await browser.pages())[0];

  console.log('READY'); // Signal Rust that the browser is ready

  rl.on('line', async (line) => {
    const [command, ...args] = line.split(' ');

    try {
      switch (command) {
        case 'GOTO':
          await page.goto(args[0]);
          console.log('SUCCESS');
          break;
        case 'EXECUTE_JS':
          const result = await page.evaluate(args.join(' '));
          console.log(`SUCCESS ${JSON.stringify(result)}`);
          break;
        case 'CLOSE':
          await browser.close();
          process.exit(0);
          break;
        default:
          console.log(`ERROR Unknown command: ${command}`);
      }
    } catch (e) {
      console.log(`ERROR ${e.message}`);
    }
  });
})();
