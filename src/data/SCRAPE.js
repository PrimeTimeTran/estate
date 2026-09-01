function getDifficulty() {
	return ['easy', 'medium', 'hard'].find((level) =>
		document.querySelector(`.text-difficulty-${level}`),
	)
}
getDifficulty()
function grabTestCases2() {
	return [...document.querySelectorAll('div.example-block')].map(
		(pre, index) => {
			const strongs = [...pre.querySelectorAll('strong')]

			const getSection = (label) => {
				const strong = strongs.find(
					(el) => el.textContent.trim() === `${label}:`,
				)

				if (!strong) return null

				const parts = []
				let node = strong.nextSibling

				while (node) {
					if (
						node.nodeType === Node.ELEMENT_NODE &&
						node.tagName === 'STRONG'
					) {
						break
					}

					parts.push(node.textContent)
					node = node.nextSibling
				}

				return parts.join('').trim()
			}

			return {
				index,
				input: getSection('Input'),
				output: getSection('Output'),
				explanation: getSection('Explanation'),
			}
		},
	)
}
grabTestCases2()
function grabTestCases() {
	return [...document.querySelectorAll('pre')].map((pre, index) => {
		const strongs = [...pre.querySelectorAll('strong')]

		const getSection = (label) => {
			const strong = strongs.find((el) => el.textContent.trim() === `${label}:`)

			if (!strong) return null

			const parts = []
			let node = strong.nextSibling

			while (node) {
				if (node.nodeType === Node.ELEMENT_NODE && node.tagName === 'STRONG') {
					break
				}

				parts.push(node.textContent)
				node = node.nextSibling
			}

			return parts.join('').trim()
		}

		return {
			index,
			input: getSection('Input'),
			output: getSection('Output'),
			explanation: getSection('Explanation'),
		}
	})
}
grabTestCases()
function copyProblemWithDesc() {
	const normalize = (s) => s.replace(/\s+/g, ' ').trim()
	const link = [...document.querySelectorAll('a[href]')].find((a) =>
		/^\d+\.\s+/.test(normalize(a.textContent)),
	)

	if (!link) {
		throw new Error('Could not find problem link')
	}

	const title = normalize(link.textContent)
	const url = new URL(link.getAttribute('href'), location.origin).href
	const number = Number(title.match(/^\d+/)?.[0])

	// Description
	const description = document.querySelector(
		'[data-track-load="description_content"]',
	)

	const paragraphs = description
		? [...description.querySelectorAll('p')].slice(0, 2)
		: []

	const descriptionHtml = paragraphs.map((p) => p.outerHTML).join('\n')
	const descriptionText = paragraphs
		.map((p) => normalize(p.textContent))
		.join('\n\n')

	const normalizedName = new URL(url).pathname.split('/').filter(Boolean).pop()

	const testCases = grabTestCases()

	const finalTestCases = testCases?.length > 0 ? testCases : grabTestCases2()

	const result = {
		number,
		'normalized-name': normalizedName,
		title: title.split(' ').slice(1).join(' '),
		difficulty: getDifficulty(),
		description: descriptionText,
		url,
		filename: `${number}-${normalizedName}.py`,
		'description-raw': descriptionHtml,
		'test-cases': finalTestCases,
	}

	const text = JSON.stringify(result, null, 2)

	const textarea = document.createElement('textarea')
	textarea.value = text
	textarea.style.position = 'fixed'
	textarea.style.opacity = '0'

	document.body.appendChild(textarea)
	textarea.select()
	document.execCommand('copy')
	textarea.remove()

	console.log(text)
}
copyProblemWithDesc()
