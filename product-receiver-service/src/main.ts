//src/main.ts
import express from 'express';
import router from './routes';

const app = express();
const port = process.env.PORT || 3002;

app.use(express.json());

app.use('/api', router);

app.listen(port, () => {
	console.log(`Product Receiver Service is running on port ${port}`);
});
