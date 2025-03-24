import { invoke } from "@tauri-apps/api/core";
import { Button, Col, Drawer, Flex, InputNumber, message, Row, Spin } from "antd";
import TextArea from "antd/es/input/TextArea";
import { useState } from "react";

interface Value {
    key: string;
    value: string | undefined;
}

export const DeleteContents = () => {
    const [loading, setLoading] = useState(false);
    const [targetN, setTargetN] = useState(3);
    const [targetPaths, setTargetPaths] = useState("");

    // 解析結果
    const [result, setResult] = useState("");
    const [resultN, setResultN] = useState(0);
    const [resultPaths, setResultPaths] = useState<string[]>([]);

    // 処理結果
    const [deletedResult, setDeletedResult] = useState("");
    const [deletedResultN, setDeletedResultN] = useState(0);
    const [deletedSuccessPaths, setDeletedSuccessPaths] = useState<Value[]>([]);
    const [deletedErrorPaths, setDeletedErrorPaths] = useState<Value[]>([]);
    const [reportPath, setReportPath] = useState("");

    const funcAnalytics = async () => {
        setLoading(true);
        try {
            const res: any = await invoke("function_analytics", { n: targetN, paths: targetPaths });
            console.log(res);

            setResult(res.message);
            setResultN(res.delete_count as number);
            setResultPaths(res.paths as string[]);

            message.success("解析が完了しました。");
        } catch (e) {
            console.error(e);
            message.error("処理中にエラーが発生しました。");
        } finally {
            setLoading(false);
        }
    };

    const funcDelete = async () => {
        setLoading(true);
        try {
            // 処理実行中に待機時間を表示するためのカウントを追加
            wait();

            const res: any = await invoke("function_ndelete", { n: resultN, paths: resultPaths });
            console.log(res);
            setDeletedResult(res.message);
            setDeletedResultN(res.delete_count as number);
            const successPathsArray = Object.entries(res.success_paths).map(([key, value]) => ({ key: key, value: value } as Value));
            setDeletedSuccessPaths(successPathsArray);
            const errorPathsArray = Object.entries(res.error_paths).map(([key, value]) => ({ key: key, value: value } as Value));
            setDeletedErrorPaths(errorPathsArray);
            setReportPath(res.report_path as string);
            message.success("削除処理が完了しました。");
        } catch (e) {
            console.error(e);
            message.error("処理中にエラーが発生しました。");
        } finally {
            setLoading(false);
        }
    };

    const [waitSecondForView, setWaitSecondForView] = useState(0);
    // isLoadingがtrueの間、待機を行い、時間を表示するための関数
    const wait = async () => {
        if (loading) {
            setWaitSecondForView(waitSecondForView + 1);
            await new Promise((resolve) => setTimeout(resolve, 1000));
            await wait();
        }
    };

    const [open, setOpen] = useState(false);
    const onOpen = () => {
        console.log("open");
        setOpen(true);
    };

    const onClose = () => {
        console.log("close");
        setOpen(false);
    };


    return (
        <>
            <Flex justify="flex-end">
                <Button type="text" onClick={onOpen}>使用方法</Button>
            </Flex>
            <Drawer title="削除対象のパス" placement="right" closable={true} onClose={onClose} open={open}>
                <h2>使用方法</h2>
                <p>対象のディレクトリまたはファイルのパスを、絶対パスで入力してください。</p>
                <p>複数のパスを入力する場合は、改行で区切ってください。</p>
                <p>「解析」ボタンをクリックすると、対象ファイルの内容が表示されます。</p>
                <p>内容を確認し、問題がなければ「削除実行」ボタンをクリックしてください。</p>
                <h2>解析について</h2>
                <p>解析では、対象ファイルに乱数データを使って複数回上書き処理を行い、その後ファイルを削除します。これにより、データの追跡や復元が不可能となります。</p>
                <p>指定したパスがディレクトリの場合、そのディレクトリ内のファイルが対象となります。ただし、ディレクトリ自体は削除されません。</p>
                <h2>削除処理について</h2>
                <p>削除処理完了後、各ファイルの処理結果が表示されます。削除に失敗した場合は、権限やファイルの状態を確認の上、再度実行してください。</p>
                <h2>リザルトファイルについて</h2>
                <p>削除処理の結果はリザルトファイルとして出力されます。</p>
                <p>このファイルには、削除が成功したファイルと失敗したファイルが記録されます。結果に示されたファイルパスに保存されるので、必要に応じてご参照ください。</p>
            </Drawer>
            {
                /* ローディング中はスピナーを表示 */
                // 待機時間を表示するためのカウント
                loading ? <p>処理中: {waitSecondForView}秒</p> : null
            }

            <Spin spinning={loading}>
                <TextArea rows={15} onChange={
                    (e) => {
                        setTargetPaths(e.target.value);
                    }
                } />

                <Row style={{ padding: '2rem', display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
                    <Flex justify="space-evenly" align="center" style={{ width: '50%' }}>
                        <Col span={12}>
                            削除回数
                        </Col>
                        <Col span={12}>
                            <InputNumber defaultValue={targetN} onChange={
                                (e) => {
                                    setTargetN(e as number);
                                }
                            } min={3} max={32} />
                        </Col>
                    </Flex>
                </Row>

                <Button onClick={
                    () => {
                        funcAnalytics();
                    }
                }>解析</Button>


                {
                    result ?
                        <>
                            <Row style={{ padding: '2rem', textAlign: 'left' }}>
                                <Col span={24}>
                                    <h2>解析結果</h2>
                                    <p>{result}</p>
                                    <p>削除回数: {resultN}</p>
                                    <p>パス</p>
                                    <ul>
                                        {
                                            // あれば表示、なければ表示しない
                                            resultPaths.length != 0 ? resultPaths.map((p, i) => {
                                                return <li key={i}>{p}</li>;
                                            }) : <><p>削除対象がありません。</p></>
                                        }
                                    </ul>
                                </Col>
                            </Row>
                            <Row>
                                <Col span={24}>
                                    <Button type="primary" danger onClick={funcDelete}>
                                        削除実行
                                    </Button>
                                </Col>
                            </Row>
                        </>
                        : null
                }

                {
                    // 削除結果
                    deletedResult ?
                        <>
                            <Row style={{ padding: '2rem', textAlign: 'left' }}>
                                <Col span={24}>
                                    <h2>削除結果</h2>
                                    <p>{deletedResult}</p>
                                    <p>削除回数: {deletedResultN}</p>
                                    <p>削除成功パス</p>
                                    <ul>
                                        {
                                            deletedSuccessPaths.map((success, index) => {
                                                return <li key={index}>{success.key}: {success.value}</li>;
                                            })
                                        }
                                    </ul>
                                    <p>削除失敗パス</p>
                                    <ul>
                                        {
                                            deletedErrorPaths.map((error, index) => (
                                                <li key={index}>{error.key}: {error.value}</li>
                                            ))
                                        }
                                    </ul>
                                    <p>Report Path: {reportPath}</p>
                                </Col>
                            </Row>
                        </>
                        : null
                }
            </Spin>
        </>
    );
}