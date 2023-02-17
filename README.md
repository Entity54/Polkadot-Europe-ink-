![license](https://img.shields.io/badge/License-Apache%202.0-blue?logo=apache&style=flat-square)

## Please use at your own risk. This a submission is for Polakdot Europ Hackathon February 2023. It is not production/battle-tested and audited software.

<br>

# **Smart Pay**

<br>

## Table of Contents

1. [General Info](#general-info)
2. [Repos](#repos)
3. [Website](#website)
4. [Demo Video](#demo-video)
5. [Installation](#installation)
6. [Using the Front End](#using-the-front-end)
7. [Business Logic](#business-logic)
8. [Smart Contracts Logic](#smart-contracts-logic)
   - [Treasury (Alias for Pallet)](#treasury)
   - [SmartPay (Alias for Treasury Manager)](#smart-pay)
   - [Factory](#factory)
   - [Oracle DEX](#oracle-dex)
   - [DOT USDT PSP22 Tokens](#psps22-tokens)

<br>

## General Info

---

<p>

The Smart Pay project comprises of tow github repos.

The current one contains all ink! smart contracts raw files (lib.rs / Cargo.toml) so the user can build and deploy these. It also contains the builds with the relevant metadata.json, _.wasm and _.contract in ink!\_smart_contracts folder for quicy deplpoyment and/or instantiation

All ink! smart contracts are been deployed on Shibuya testnet

The second repo at https://github.com/Entity54/PolkadotEurope-FrontEnd contains a create-react-app front end showing the most important features about the smart contracts. The user can also see all features and examine in great detail all data and functions using the https://contracts-ui.substrate.io

Note: By design, only the Admin account and Manager of the ink! smart contracts can interact with these. Details of these account are shown shorlty

## TODO ADD ACCOUNT FILES HERE and link to folder

<br>

## Repos

---

ink! smart contracts

https://github.com/Entity54/Polkadot-Europe-ink-

<br>

Front End

https://github.com/Entity54/PolkadotEurope-FrontEnd

<br>

## Website

---

<p>
We have deployed a website 
<a href="https://smart-pay-gamma.vercel.app/react/demo/smartpaydashboard" target="_blank">here</a>

https://smart-pay-gamma.vercel.app/react/demo/smartpaydashboard

to showcase our application and hackathonsubmission.

</p>
<p>
Please make sure you have the Polkadot wallet chrome extension installed.
</p>

### Demo Video

---

<p> A demo video can be found 
<a href="https://www.youtube.com/watch?v=LbPTr55i6kA" target="_blank">here</a>

https://www.youtube.com/watch?v=LbPTr55i6kA
to demonstrate the many features of our XCM application. Please check it out!

### Installation

For ink! smart contract ink! 3.4.0 and openbrush v2.3.0 are being used

---

We assume the user has followed the official Substrate documentation to install rust, ink! and substrate-contracts-node

<br>

For the front end

---

Create a new folder and inside it

```bash
$ git clone https://github.com/Entity54/PolkadotEurope-FrontEnd
$ cd PolkadotEurope-FrontEnd
$ yarn
$ npm start
```

### Using The Front End

---

<p>
Please see the relevant repo readme here https://github.com/Entity54/PolkadotEurope-FrontEnd
<p>

<br>

## Business Logic

---

SmartPay is an application designed to give projects multiple options when making payments from their Treasury accounts.

SmartPay was born from the real-world problems and inefficiencies that project teams face when making payments that have been approved through their Treasury processes.

As it currently stands, anyone applying for funding from a project Treasury normally include an estimation of the number of tokens they require to complete their proposal and this estimation is based on either the current or recent average price of the token against their fiat base currency they need to spend.

There are many fundamental problems with this approach, including:

- The token/fiat price at the time of submitting the proposal can often be wildly different to the price at the time the proposal is eventually passed and the funds distributed. If the applicant requires fiat to complete the proposal this can result in there being either a shortfall or an overpayment by the treasury to the applicant.
  <br>
  If there is a shortfall, due to the token/fiat price depreciating during the Treasury approval process then the applicant often has to submit a follow-up Treasury proposal for a “Top Up” so they have all the fiat funds they originally required. These “Top Up’ requests are always granted by the Treasuries making the process very inefficient.
  <br>
  If the token/fiat price appreciates during the Treasury approval process, this results in the applicant receiving more fiat than they needed when they execute their swap, sometime considerably more. The Treasury could have saved many tokens if they had the ability to use an upto date token/fiat at the moment of distribution instead.

- There are many Treasury proposals whereby the applicant does not require the funds until a date in the future, regardless of how long the Treasury approval process takes. These payments, whether they are either in the native token or a fiat-denominated amount of the native token, can be held in the SmartPay smart contracts until the future date, at which point any token/fiat calculations can be made before automatic distribution.
  <br>
  There are scenarios whereby successful Treasury proposals do not need to be transferred to the applicant at all. A good example of this would be whereby a company applies to organise a hackathon.
  <br>
  To be efficient, the proposal should ideally be split into two parts, the first being the funds needed to organise the hackathon and the second being the prize monies involved. There is no real reason that the prize money allocations should be transferred to the hackathon organisers at any point. Aside from the trust involved in making this larger type of payment to an organiser (often months before prizes are due), there is also the price volatility problem mentioned above as many Hackathon prizes are denominated in fiat amounts.
  <br>
  It is entirely possible that some large proposals take advantage of the way the system currently works by hoping for the token/fiat price to appreciate to receive more money than needed, whilst using the guaranteed “Top Up” process as a backstop.

- There are often times when specialists from different ecosystems can help one another but they don’t wish to receive the native currency of that ecosystem. The SmartPay application also solves this friction by allowing the Treasury proposal to be settled in an alternative token, on a future date, making use of various oracles and a dex.
  <br>
  Smartpay distributions can also take the form of Interval Payments. This is especially useful if an applicant makes a Treasury proposal for work that involves ongoing, regular payments. Examples might include payments for regular articles, translations, meetups etc. Whilst the Treasury would transfer the complete funds to SmartPay upon approval, the payments to the applicant would be made on the agreed regular future dates and at the prevailing token/fiat rate (if they were fiat denominated).
  <br>
  By using the SmartPay system, the project Treasuries have access to the latest information regarding the amount of payments to be released on a 2 Day, 7 Day, 30 Day and Total basis.
  <br>
  By using this approach SmartPay can query the average price via the oracle to make sure that it can safely meet the fiat denominated distributions on those time horizons.
  <br>
  If the system detects a shortfall on the funds due to be distributed, due to the token/fiat rate depreciating in value, a notification is sent to the Treasury to send more funds. This could be in the form of an expedited Treasury proposal in future versions.
  <br>
  Conversely, if the tokens held become in excess of the funds to be distributed, due to the token/fiat rate appreciating then a notification can be sent to the Treasury to inform them that excess token can be withdrawn making the whole system efficient.

<br>
<br>
<br>
<br>

6. [Smart Contracts Logic](#smart-contracts-logic)
   - [DOT USDT PSP22 Tokens](#psps22-tokens)
   - [Oracle DEX](#oracle-dex)
   - [Factory](#factory)
   - [Treasury (Alias for Pallet)](#treasury)
   - [SmartPay (Alias for Treasury Manager)](#smart-pay)

## Smart Contracts Logic

---

We assume that there is a treasury smart contract or a pallet like in Polkadot Treasury. The crux of the matter is that this is where successful referendums that end up in funding jobs end up. To re-iterate every job submitted in treasury smart contract is considered approved.

The factory smart contract has the sole job of allowing any one (e.g. any treasury smart contract) to launch its own SmarPay which comes with certain traits and functionality

The Oracle DEX acts as a source for pricing for the SmartPay and also for executing swaps

To sum up the Treasury smart contract launches and own its own SmartPay via the factory smart contract and this SmartPay retieves prices and executes swaps via the Oracle

> Note: In out specific implementation example we have used the scenario that the Tresury smart contract (pallet) has DOT as its treasury token

<br>
<br>

## PSP22 Tokens

For the purposes of our submission we have deployed and minted DOT and USDT PSSP22 tokens. These offer all the advantages of PSP22 functionality and ERC20 equivalent behaviour (and more). It can easily be assumed that native DOT and USDT could be wrapped in PSP22

<br>

## Oracle DEX

An Oracle has been developped to provide prices and allow swappability. This is not an AMM DEX and it was built purposely designed for our payments project submission.

It shoudl be said that the Oracle still allows to register and add liquidity to pools, calculates a 5 period moving average in an efficient way, activates or pauses a pair among other features

<br>

## Factory

Allows the launch (or relaunch) of a Smart Pay (alias for treasury manager) smart contract keeping track of who is the owner (e.g. pallet)

<br>

## Tresury

The Admin of Treasury has full power on the Treasury and the Smart Pay

The Admin account can launch a Smart Pay smart contract stating whhich account will have the PSP role of MANAGER of rhe Smart Pay smart contract (in our case here it is the same account)

The Admin can deposit or withdraw DOT from his/her Smart Pay

The Admin can change the Admin and Manager role to another account

The Admin can terminate the Smart Pay smart contract

The Admin can Add a new Job to the Treasury (approved referendum) whihc will be autmatically transmitted to the Smart Pay smart contract

<br>

## Smart Pay

Each new job that arrives joins the Open Jobs vector. Once job's timestmap is surpassed by the block timestamp then the job is moved to Pending jobs.

Pending jobs are either jobs that are of type One-Off future payment that will be paid imminently or Installment type of jobs that once the 1st installment is paid will remain in Pending jobs until all installments are paid out

Jobs then are paid out and mvoe to theri final state of completed jobs

A job can be One-Off or Installment (periodic) type of payment

A job applicant can ask X amount of tokens (DOT or USDT)
A job applicant can ask the denomination to be in DOT or USDT

Examples

Applicant asks to be paid 100 DOT in value DOT (non USDT) => receives 100 DOT

Applicant asks to be paid the vbalue of 100 USD in DOT => If the 5-perion moving averga of DOT is 10 then the applicant receives 100 / 10 = 10 DOT

Applicant asks to be paid 100 USD in USDT => if the last price from the oracle for DOT/USDT is $5 then the DOT are swapped for USDT and the applicant receives 20 USDT in his account

<br>

## IMPORTANT

SMART CONTRACTS

## ADMIN ARISU2

const Arisu2_PHRASE = 'bachelor axis digital trend canyon diesel pencil giraffe noise maze rose become';  
//Arisu2 IS ADMIN
//ADDRESS 5HMwBS1bxriTYnGo6m8AFEKTaoDmTKJeMvoMpQXENJsv1RBg

<br>

## DOT

YovEh7RQkxjK6y2FKpKK8urtTofEPMdYvzQXNCFczeAqwmJ

## USDT

XiL4V7XGc6PhTMxCNtPfkx7kjD8zuR36R1MfA3pYbm7QYZD

## OracleDEXAstar

YSefjGpCV1sC9K6LGwGPiZPNyxKDTBCxRkVd7WSkEC643yD

## FACTORY_T_M

XQjsXLXyqQr4HQweWVkbx8jsnhN8LGTv7iKWWBxysNP6wor

## Pallet_Governor

Z5BbE8EA2vTPCzBCWKb9hnjbPMXTsrD3wDHh3HTMgiCiKTt

CODE_HASH FOR TREASURY MANAGER (SMARTPAY)
0x0beaa08a003b717be7fea54601173d4826baa6708955a0b3dfbf142f44fe1468

## LAUNCHED TREASURY MANAGER

YxsHyDbUvxHBqCMKpqu6xJ7A5Y8Wu5c8wywhyngXfh3f88N

ADMIN ACCOUNT FINANCES PALLET BY TRANFERRING 1_000,000,000,000,011,000,000

ARISU2
SEED
bachelor axis digital trend canyon diesel pencil giraffe noise maze rose become
Address
5HMwBS1bxriTYnGo6m8AFEKTaoDmTKJeMvoMpQXENJsv1RBg
