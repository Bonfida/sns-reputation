<h1 align="center">SNS Reputation</h1>
<br />
<p align="center">
<img width="250" src="https://camo.githubusercontent.com/7ced22029b2f005e464f15db32caaa9a56b820f1854d8571ef9093f3c481019d/68747470733a2f2f692e696d6775722e636f6d2f6e6e374c4d4e562e706e67"/>
</p>
<p align="center">
<a href="https://twitter.com/bonfida">
<img src="https://img.shields.io/twitter/url?label=Bonfida&style=social&url=https%3A%2F%2Ftwitter.com%2Fbonfida">
</a>
</p>
<br />

<h2 align="center">What is SNS Reputation?</h2>

This project is part of the [Solana Hackathon HYPERDRIVE](https://solana.com/hyperdrive).

SNS Reputation is a two-faceted initiative: Community-driven score and Algorithmic score.

<h2 align="center">Community-Driven Score</h2>

The community-driven score allows users to upvote/downvote other users based on their behaviour. This score can be used to create on-chain reputation and see which users are more trustworthy than others. There is no moderation and it's purely community driven. The vote of each user is weighted by how much Solana they stake. If users don't stake they cannot vote.

The smart contract is currently deployed on mainnet at `4X9mF1yUx2ez6ifYCmr2aYJnX5DkKAxbu5QD93s7gooG`.

Smart contract bindings are availble in Javascript and Python.

<h2 align="center">Algorithmic Score</h2>

The Algorithmic approach is here to complement the community approach. The community approach is incomplete and biased, this is why we need to provide a more systematic and quantitative solution. In order to develop a systematic approach we need to build a data set. There is no open source and public data set of suspicious address or transaction. Everything is kept private and monetized by companies. We believe there should be an open source data set available to the community to build the best models based from this and share their models. Eventually, the open source approach provides more value and should win. This is a public good we want to build. However, getting this data set is complicated and the data collection might take time. This is why the first step is to collect data from users reports.

To collect this data, we use a Cloudflare worker. This worker is responsible for gathering and processing user reports.

<h2 align="center">How to use it?</h2>

<h3 align="center">Report a wallet or transaction</h3>

To report a suspicious wallet or transaction, follow these steps:

1. Navigate to the SNS website: [https://sns.id](https://sns.id).
2. Connect your wallet and enable the beta toggle.
3. Click on the 'SNS Reputation' button located on the navigation bar.
4. In the form that appears, enter the wallet address or transaction ID that you want to report.
5. Provide any additional details in the 'Comments' section.
6. Click 'Submit' to send your report. Our Cloudflare worker will gather and process your report.



https://github.com/Bonfida/sns-reputation/assets/47689875/c88730dc-607b-4d79-89f5-453d6067ad8f



<h3 align="center">Vote on users</h3>

To vote on users based on their behavior, follow these steps:

1. Navigate to the SNS website: [https://sns.id](https://sns.id).
2. Connect your wallet and enable the beta toggle.
3. Go to the profile of the user on which you want to vote.
4. To vote, click on the 'Upvote' or 'Downvote' button next to the user's name.
5. Your vote will be weighted by how much Solana you stake. If you don't stake, you cannot vote.
6. The smart contract will handle your vote.



https://github.com/Bonfida/sns-reputation/assets/47689875/3d236540-f781-4fb6-884f-d6f4549b2936



Remember, the aim of voting is to create an on-chain reputation and see which users are more trustworthy than others. It's purely community driven and there is no moderation.
