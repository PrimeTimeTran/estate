const fs = require('fs')

const lines = fs.readFileSync(0, 'utf8').trim().split('\n')
const nums = JSON.parse(lines[0])
const target = Number(lines[1])

const seen = new Map()

for (let i = 0; i < nums.length; i++) {
	const complement = target - nums[i]

	if (seen.has(complement)) {
		console.log(JSON.stringify([seen.get(complement), i]))
		process.exit(0)
	}

	seen.set(nums[i], i)
}
