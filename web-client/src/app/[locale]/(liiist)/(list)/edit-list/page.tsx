"use client";

// app/edit-list/page.tsx
import React, { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Input, Input2 } from "@/components/ui/input";
import { TagInput } from "@/components/ui/tag-input";
import { ActionButton2 } from "@/components/ui/ActionButton";
import { ToggleSwitch } from "@/components/ui/ToggleSwitch";
import { FaMagnifyingGlassArrowRight } from "react-icons/fa6";
import { handleCalculate2 } from "@/services/shoppingListService";

const EditListPage: React.FC = () => {
    const searchParams = useSearchParams();
    const router = useRouter();
    const listId = searchParams.get("id");
    const colors = ["#FFABAD", "#FFC576", "#B4B1B1" , "#7D5C65", "#6EEB83"];
    const [listTitle, setListTitle] = useState<string>("");
    const [products, setProducts] = useState<{ name: string; quantity: number }[]>([]);
    const [budget, setBudget] = useState<string>("");
    const [mode, setMode] = useState<string>("convenience");
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);
    const [backgroundColor, setBackgroundColor] = useState("#ffffff")
    
    useEffect(() => {
        // Recuperare la lista da modificare usando l'ID della lista
        if (listId) {
            fetch(`/api/shopping-lists/${listId}`)
            .then((response) => {
                if (!response.ok) {
                    throw new Error("Failed to fetch list data");
                }
                return response.json();
            })
            .then((data) => {
                setListTitle(data.name || "");
                setProducts(data.products || []);
                setBudget(data.budget || "");
                setMode(data.mode || "convenience");
                setIsLoading(false);
                const createdDate = new Date(data.createdAt);
                const minute = createdDate.getMinutes();
                var lastDigit = minute % 10;
                if(lastDigit > 5){
                    lastDigit = lastDigit - 5;
                }
                const choosecolor = colors[lastDigit % colors.length];
                setBackgroundColor(choosecolor);
                })
                .catch((err) => {
                    setError(err.message);
                    setIsLoading(false);
                });
        }
    }, [listId]);

    const handleSaveChanges = async () => {
        setIsLoading(true);
        setError(null);
        try {
            // Modificare la lista tramite una richiesta PUT all'endpoint API
            const response = await fetch(`/api/shopping-lists/${listId}`, {
                method: "PUT",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    name: listTitle,
                    products,
                    budget,
                    mode,
                }),
            });
            if (!response.ok) {
                throw new Error("Failed to update the shopping list");
            }
            router.push("/home");
        } catch (err) {
            setError((err as Error).message);
        } finally {
            setIsLoading(false);
        }
    };

    const handleIncreaseQuantity = (index: number) => {
        const updatedProducts = [...products];
        updatedProducts[index].quantity += 1;
        setProducts(updatedProducts);
    };

    const handleDecreaseQuantity = (index: number) => {
        const updatedProducts = [...products];
        if (updatedProducts[index].quantity > 1) {
            updatedProducts[index].quantity -= 1;
            setProducts(updatedProducts);
        }
    };

    const handleToggleMode = () => {
        setMode(mode === "convenience" ? "savings" : "convenience");
    };

    return (
        <div className="max-w-full w-full flex justify-center p-5 text-liiist_black">
            {isLoading ? (
                <p>Loading...</p>
            ) : (
                <div id="edit-list-card" className="max-w-4xl w-full mt-5 flex bg-slate-50 rounded-xl shadow-md">
                    <div className="w-full flex-col flex">
                        {error && (
                            <div className="text-red-500 mb-4">{error}</div>
                        )}
                        <div className="rounded-tl-lg h-16 " style={{backgroundColor}}>
                            <Input2
                                placeholder="List Title"
                                value={listTitle || ""}
                                onChange={(e) => setListTitle(e.target.value)}
                                className="rounded-tl-lg"
                            />
                        </div>
                        <div className="mb-5 pt-4 gap-2 flex px-2 items-center border-t-2 border-dashed border-gray-500">
                            <span className="py-2 ">
                                Budget€
                            </span>
                            <Input
                                type="number"
                                placeholder="Budget"
                                value={budget || ""}
                                onChange={(e) => setBudget(e.target.value)}
                                className="border-transparent shadow-none font-medium w-1/6"
                            />
                        </div>
                        <div className="mb-5 px-2">
                            <TagInput
                                placeholder="Add product"
                                onAdd={(product) =>
                                    setProducts([...products, product])
                                }
                                onRemove={(index) => {
                                    const updatedProducts = [...products];
                                    updatedProducts.splice(index, 1);
                                    setProducts(updatedProducts);
                                }}
                                onIncreaseQuantity={handleIncreaseQuantity}
                                onDecreaseQuantity={handleDecreaseQuantity}
                                tags={products || []}
                            />
                        </div>
                        <div className="mb-5 pl-2">
                            <ToggleSwitch
                                checked={mode === "savings"}
                                onChange={handleToggleMode}
                                labels={["Convenience", "Savings"]}
                            />
                        </div>
                    </div>
                    <div className="w-1/6 h-full border-l-2 border-dashed border-gray-500 flex flex-col justify-between items-center">
                        <ActionButton2
                            onClick={() =>
                                handleCalculate2(
                                    listId,
                                    listTitle,
                                    products,
                                    budget,
                                    mode,
                                    "12345",
                                    router
                                )
                            }
                            disabled={isLoading || products.length === 0 || budget === ""}
                            className=" rounded-full flex justify-center"
                            >
                            <FaMagnifyingGlassArrowRight className="text-3xl hover:scale-125"/>
                        </ActionButton2>
                        <ActionButton2
                            onClick={handleSaveChanges}
                            disabled={isLoading}
                            className={"rounded-full text-center hover:scale-125 w-full"}
                        >
                        {isLoading ? "Saving..." : "Save"}
                        </ActionButton2>
                    </div>
                </div>
            )}
        </div>
    );
};

export default EditListPage;