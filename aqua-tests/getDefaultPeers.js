// @ts-check
const {
  krasnodar,
  stage,
  testNet,
} = require('@fluencelabs/fluence-network-environment')

const networksMap = {
  krasnodar,
  stage,
  testnet: testNet,
}

const networksString = Object.keys(networksMap).join(', ')

const maybeNetworkArg = process.argv[3]

if (maybeNetworkArg === undefined) {
  console.error(`Expected at least one argument (${networksString})`)
  process.exit(1)
}

const networkArg = maybeNetworkArg
const maybeNodes = networksMap[networkArg]

if (maybeNodes === undefined) {
  console.error(`Invalid argument. Expected one of: ${networksString}`)
  process.exit(1)
}

const nodes = maybeNodes
console.log(nodes.map(({ multiaddr }) => multiaddr).join('\n'))
