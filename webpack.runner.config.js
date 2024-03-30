const webpack = require('webpack')
const path = require('path')
const { copyFile, readFile } = require('node:fs/promises');

const entrypoint = "server/job-reader.ts"
async function setEnvironmentFromProcess() {
    const environments = [ "local", "stage", "service" ]
    process.env.NODE_ENV = (process.env.NODE_ENV ?? "").toLowerCase()

    if (!environments.includes(process.env.NODE_ENV)) {
        throw new Error("Invalid deploy environment provided")
    }

    let confPath = path.resolve("./.env.d/" + process.env.NODE_ENV);

    return copyFile(confPath, path.resolve("./.env"))
    .then(async () => {
        const contents = await readFile(confPath, {encoding: "utf-8"})
        let vars = contents.split(/\n/).map(v => [v.slice(0, v.indexOf("=")), v.slice(1+ v.indexOf("="))])
        vars.forEach(pair => {
            process.env[pair[0]] = pair[1]
        })
        return vars
    })
    .catch((e) => { console.log("Error while setting application environment variables"); console.log(e); process.exit(1) })
}
class SetEnvironment {
    apply(compiler) {
        compiler.hooks.beforeRun.tap('SetFriendsEnvironment', async (compiler) => {
            let vs = await setEnvironmentFromProcess()
        });
    }
}
module.exports = {
    devtool: 'inline-source-map',
    entry: {
        app: { 
            import: path.resolve(__dirname, entrypoint), 
            filename: process.env.NODE_ENV + "-wendy-runner.js" 
        }
    },
    mode: process.env.NODE_ENV == "service" ? 'production' : 'development',
    module: {
        rules: [
            {
                test: /\.ts$/,
                exclude: [/node_modules/],
                loader: 'ts-loader',
                options: {
                    configFile: "tsconfig-runner.json"
                }
            }
        ]
    },
    resolve: { 
        modules: [ path.resolve('./node_modules') ],
        extensions: ['.ts', '.js'] 
    },
    output: {
        chunkFilename: '[name].js',
        filename: '[name].js'
    },
    plugins: [
        new webpack.IgnorePlugin({resourceRegExp: /\/index\.html$/}),
        new webpack.IgnorePlugin({resourceRegExp: /.cs$/}),
        new SetEnvironment()
    ],
    target: 'node',
}
