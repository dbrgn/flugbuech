interface Data {
    /**
     * Number of registered users.
     */
    readonly userCount: number;
    /**
     * Number of registered gliders.
     */
    readonly gliderCount: number;
    /**
     * Total flights in database.
     */
    readonly flightCount: number;
}

export async function load(): Promise<Data> {
    return {
        userCount: 0,
        gliderCount: 0,
        flightCount: 0,
    };
}
