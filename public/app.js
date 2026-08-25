console.log('APP.JS LOADED')
console.log('APP.JS LOADED LOI')

window.js_test = function (payload) {
	console.log('JS RECEIVED FROM RUST:', payload)

	payload.name = 'Modified by JavaScript'
	payload.count = 999

	console.log('JS MODIFIED:', payload)
}
