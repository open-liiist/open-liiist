import { configSchema, Config } from '../schemas/configSchema'
import merge from 'lodash.merge' // Aggiungi lodash.merge per un merge profondo

export const configFactory = (overrides: Partial<Config>): Config => {
    const envConfig = {
        appName: process.env.APP_NAME,
        isProduction: process.env.NODE_ENV === 'production',
        isDevelopment: process.env.NODE_ENV === 'development',
        isTest: process.env.NODE_ENV === 'test',
        cookies: {
            access: process.env.NAME_COOKIE_ACCESS,
            refresh: process.env.NAME_COOKIE_REFRESH,
        },
        appPort: process.env.APP_PORT ? parseInt(process.env.APP_PORT, 10) : 3000,
        apiBaseUrl: process.env.NEXT_PUBLIC_API_BASE_URL, // Usa NEXT_PUBLIC_API_BASE_URL
        frontendUrl: process.env.FRONTEND_URL,
        jwtSecret: process.env.JWT_SECRET,
        jwtExpiration: process.env.JWT_EXPIRATION
    }

    // Utilizza lodash.merge per un merge profondo
    const mergedConfig = merge({}, envConfig, overrides)

    const validatedConfig = configSchema.safeParse(mergedConfig)

    if (!validatedConfig.success) {
        throw new Error(
            'Invalid configuration: ' + JSON.stringify(validatedConfig.error.format())
        )
    }

    return validatedConfig.data
}
