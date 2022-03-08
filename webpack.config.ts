import path from "path";
import { Configuration, DefinePlugin } from "webpack";
import TsconfigPathsPlugin from "tsconfig-paths-webpack-plugin";

const webpackConfig = (): Configuration => ({
	entry: {
		"index": "./ui/index.ts"
	},
	...(process.env.production || !process.env.development
		? {}
		: { devtool: "eval-source-map" }),
	resolve: {
		extensions: [".ts", ".tsx", ".js"],
		plugins: [new TsconfigPathsPlugin({ configFile: "./tsconfig.json" })],
	},
	output: {
		path: path.join(__dirname, "/dist"),
		filename: "[name].js",
	},
	module: {
		rules: [
			{
				test: /\.tsx?$/,
				loader: "ts-loader",
				options: {
					transpileOnly: true,
				},
				exclude: /dist/,
			},
			{
				test: /\.s?css$/,
				use: ["style-loader", "css-loader", "sass-loader"],
			},
		],
	},
	// devServer: {
	// 	port: 3000,
	// 	open: true,
	// 	historyApiFallback: true,
	// },
	plugins: [
		// DefinePlugin allows you to create global constants which can be configured at compile time
		new DefinePlugin({
			"process.env": process.env.production || !process.env.development,
		}),
	],
});

export default webpackConfig;