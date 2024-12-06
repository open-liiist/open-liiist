import { Noto_Sans_JP} from 'next/font/google';
import { Nunito } from 'next/font/google';
import {Oswald} from 'next/font/google';

export const noto_Sans = Noto_Sans_JP({
	subsets: ['vietnamese'],
	variable: '--font-noto_Sans'
});

export const nunito = Nunito({
	subsets: ['latin']
});

export const oswald = Oswald({
	subsets: ['latin']
});