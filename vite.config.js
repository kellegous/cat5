import { defineConfig } from 'vite'
export default defineConfig({
	root: './ui',
	build: {
		outDir: '../dist',
		assetsDir: '',
		rollupOptions: {
			input: {
				main: './ui/index.html',
			},
		},
	}
});